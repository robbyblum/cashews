use std::pin::Pin;

use futures::Stream;
use sea_query::{Asterisk, Expr, PostgresQueryBuilder, Query, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    ChronDb, Idens,
    models::{EntityKind, EntityObservation, EntityVersion, HasPageToken, PageToken},
};

pub struct GetEntitiesQuery {
    pub kind: EntityKind,
    pub at: Option<OffsetDateTime>,
    pub id: Vec<String>,
    pub count: u64,
    pub order: SortOrder,
    pub page: Option<PageToken>,
    pub before: Option<OffsetDateTime>,
    pub after: Option<OffsetDateTime>,
}

pub struct GetVersionsQuery {
    pub kind: EntityKind,
    pub id: Vec<String>,
    pub before: Option<OffsetDateTime>,
    pub after: Option<OffsetDateTime>,
    pub count: u64,
    pub order: SortOrder,
    pub page: Option<PageToken>,
}

impl ChronDb {
    pub async fn get_all_entity_ids(&self, kind: EntityKind) -> anyhow::Result<Vec<String>> {
        let ids = sqlx::query_scalar("select entity_id from latest_versions where kind = $1")
            .bind(kind)
            .fetch_all(&self.pool)
            .await?;
        Ok(ids)
    }

    pub async fn get_all_entity_ids_slow(&self, kind: EntityKind) -> anyhow::Result<Vec<String>> {
        let ids = sqlx::query_scalar("select distinct entity_id from observations where kind = $1")
            .bind(kind)
            .fetch_all(&self.pool)
            .await?;
        Ok(ids)
    }

    pub async fn get_all_latest(&self, kind: EntityKind) -> anyhow::Result<Vec<EntityVersion>> {
        let res = sqlx::query_as("select kind, entity_id, valid_from, null as valid_to, data from latest_versions inner join objects using (hash) where kind = $1")
            .bind(kind)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }

    pub async fn get_latest(
        &self,
        kind: EntityKind,
        entity_id: &str,
    ) -> anyhow::Result<Option<EntityVersion>> {
        let res = sqlx::query_as("select kind, entity_id, valid_from, null as valid_to, data from latest_versions inner join objects using (hash) where kind = $1 and entity_id = $2")
            .bind(kind)
            .bind(entity_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }

    pub async fn get_entity_at(
        &self,
        kind: EntityKind,
        entity_id: &str,
        timestamp: &OffsetDateTime,
    ) -> anyhow::Result<Option<EntityVersion>> {
        let res = sqlx::query_as("select kind, entity_id, valid_from, null as valid_to, data from versions inner join objects using (hash) where kind = $1 and entity_id = $2 and $3 >= valid_from and $3 < coalesce(valid_to, 'infinity')")
            .bind(kind)
            .bind(entity_id)
            .bind(timestamp)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }

    pub async fn get_latest_observation(
        &self,
        kind: EntityKind,
        entity_id: &str,
    ) -> anyhow::Result<Option<EntityObservation>> {
        let res = sqlx::query_as("select kind, entity_id, timestamp, data from observations inner join objects using (hash) where kind = $1 and entity_id = $2")
            .bind(kind)
            .bind(entity_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }

    pub async fn get_entities(
        &self,
        q: GetEntitiesQuery,
    ) -> anyhow::Result<PaginatedResult<EntityVersion>> {
        let mut qq = Query::select()
            .expr(Expr::col((Idens::Versions, Asterisk)))
            .expr(Expr::col(Idens::Data))
            .from(Idens::Versions)
            .inner_join(
                Idens::Objects,
                Expr::col((Idens::Versions, Idens::Hash)).equals((Idens::Objects, Idens::Hash)),
            )
            .order_by_columns([
                (Idens::ValidFrom, get_order(q.order)),
                (Idens::EntityId, get_order(q.order)),
            ])
            .limit(q.count)
            .and_where(Expr::col(Idens::Kind).eq(q.kind as i32))
            .to_owned();

        if !q.id.is_empty() {
            qq = qq
                .and_where(Expr::col(Idens::EntityId).is_in(q.id))
                .to_owned();
        }

        if let Some(at) = q.at {
            qq = qq
                .and_where(Expr::val(at).gte(Expr::col(Idens::ValidFrom)))
                .and_where(Expr::val(at).lt(Expr::cust_with_expr(
                    "coalesce($1, 'infinity')",
                    Expr::col(Idens::ValidTo),
                )))
                .to_owned();
        } else {
            qq = qq.and_where(Expr::col(Idens::ValidTo).is_null()).to_owned();
        }

        if let Some(page) = q.page {
            qq = qq
                .and_where(paginate(
                    q.order,
                    Idens::ValidFrom,
                    Some(Idens::EntityId),
                    page,
                ))
                .to_owned();
        }

        if let Some(before) = q.before {
            qq = qq
                .and_where(Expr::col(Idens::ValidFrom).lte(before))
                .to_owned();
        }

        if let Some(after) = q.after {
            qq = qq
                .and_where(Expr::col(Idens::ValidFrom).gte(after))
                .to_owned();
        }

        let (q, vals) = qq.build_sqlx(PostgresQueryBuilder);
        let res = sqlx::query_as_with(&q, vals).fetch_all(&self.pool).await?;
        Ok(with_page_token(res))
    }

    pub async fn get_versions(
        &self,
        q: GetVersionsQuery,
    ) -> anyhow::Result<PaginatedResult<EntityVersion>> {
        let mut qq = Query::select()
            .expr(Expr::col((Idens::Versions, Asterisk)))
            .expr(Expr::col(Idens::Data))
            .from(Idens::Versions)
            .order_by_columns([
                (Idens::ValidFrom, get_order(q.order)),
                (Idens::EntityId, get_order(q.order)),
            ])
            .limit(q.count)
            .inner_join(
                Idens::Objects,
                Expr::col((Idens::Versions, Idens::Hash)).equals((Idens::Objects, Idens::Hash)),
            )
            .and_where(Expr::col(Idens::Kind).eq(q.kind as i32))
            .to_owned();

        if !q.id.is_empty() {
            qq = qq
                .and_where(Expr::col(Idens::EntityId).is_in(q.id))
                .to_owned();
        }

        if let Some(before) = q.before {
            qq = qq
                .and_where(Expr::col(Idens::ValidFrom).lte(before))
                .to_owned();
        }

        if let Some(after) = q.after {
            qq = qq
                .and_where(Expr::col(Idens::ValidFrom).gte(after))
                .to_owned();
        }

        if let Some(page) = q.page {
            qq = qq
                .and_where(paginate(
                    q.order,
                    Idens::ValidFrom,
                    Some(Idens::EntityId),
                    page,
                ))
                .to_owned();
        }

        let (q, vals) = qq.build_sqlx(PostgresQueryBuilder);
        let res = sqlx::query_as_with(&q, vals).fetch_all(&self.pool).await?;
        Ok(with_page_token(res))
    }

    pub async fn get_version_count(&self, kind: EntityKind) -> anyhow::Result<usize> {
        let res: i64 = sqlx::query_scalar("select count(*) from versions where kind = $1")
            .bind(kind)
            .fetch_one(&self.pool)
            .await?;

        Ok(res as usize)
    }

    pub async fn get_all_versions_stream(
        &self,
        kind: EntityKind,
    ) -> anyhow::Result<
        Pin<Box<dyn Stream<Item = sqlx::Result<EntityVersion, sqlx::Error>> + Send + '_>>,
    > {
        let res = sqlx::query_as::<_, EntityVersion>("select kind, entity_id, valid_from, valid_to, data from versions inner join objects using (hash) where kind = $1 order by valid_from")
            .bind(kind)
            .fetch(&self.pool);

        Ok(res)
    }
}

pub fn paginate(
    order: SortOrder,
    timestamp_col: Idens,
    id_col: Option<Idens>,
    page_token: PageToken,
) -> SimpleExpr {
    let (ls, rs) = if let Some(id_col) = id_col {
        let ls = Expr::tuple([Expr::col(timestamp_col).into(), Expr::col(id_col).into()]);
        let rs = Expr::tuple([
            Expr::value(page_token.timestamp),
            Expr::value(page_token.entity_id),
        ]);
        (ls, rs)
    } else {
        (Expr::col(timestamp_col), Expr::val(page_token.timestamp))
    };

    match order {
        SortOrder::Asc => ls.gt(rs),
        SortOrder::Desc => ls.lt(rs),
    }
}

pub fn paginate_simple(order: SortOrder, id_col: Idens, page_token: PageToken) -> SimpleExpr {
    let (ls, rs) = {
        let ls = Expr::col(id_col);
        let rs = Expr::value(page_token.entity_id);
        (ls, rs)
    };

    match order {
        SortOrder::Asc => ls.gt(rs),
        SortOrder::Desc => ls.lt(rs),
    }
}

pub fn with_page_token<T: HasPageToken>(items: Vec<T>) -> PaginatedResult<T> {
    let pt = items.last().map(|e| e.page_token());
    PaginatedResult {
        items,
        next_page: pt,
    }
}

pub fn get_order(order: SortOrder) -> sea_query::Order {
    match order {
        SortOrder::Asc => sea_query::Order::Asc,
        SortOrder::Desc => sea_query::Order::Desc,
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Asc
    }
}

#[derive(Serialize, Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub next_page: Option<PageToken>,
}
