use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use brotli::CompressorWriter;
use dashmap::DashMap;
use tracing::{error, info};
use uuid::Uuid;

use super::WorkerContext;
use futures::StreamExt;

pub async fn crunch(ctx: &WorkerContext) -> anyhow::Result<()> {
    // leftover code from an experiment, ignore this

    sqlx::query("truncate table objects_compressed_cbor_br_9_intkeys")
        .execute(&ctx.db.pool)
        .await?;

    let stream = sqlx::query_as::<_, (Uuid, serde_json::Value)>("select hash, data from objects")
        .fetch(&ctx.db.pool);

    let key_map = Arc::new(DashMap::new());
    let next = Arc::new(Mutex::new(0));

    stream
        .map(move |x| do_crunch(ctx, x, Arc::clone(&key_map), Arc::clone(&next)))
        .buffer_unordered(500)
        .filter_map(async |x| x.ok())
        .chunks(100)
        .map(|x| submit(ctx, x))
        .buffer_unordered(20)
        .enumerate()
        .for_each(|(i, res)| async move {
            if let Err(e) = res {
                error!("error processing item: {:?}", e);
            } else if i % 1 == 0 {
                info!("{} in", i);
            }
        })
        .await;
    Ok(())
}

async fn do_crunch(
    _ctx: &WorkerContext,
    row: sqlx::Result<(Uuid, serde_json::Value)>,
    key_map: Arc<DashMap<String, u32>>,
    next: Arc<Mutex<u32>>,
) -> anyhow::Result<(Uuid, Vec<u8>)> {
    let (hash, value) = row?;

    let buf = tokio::task::spawn_blocking(move || encode(value, &key_map, &next)).await?;
    Ok((hash, buf))

    // sqlx::query(
    //     "insert into objects_compressed_json_zstd (hash, data) values ($1, $2) on conflict do nothing",
    // )
    // .bind(hash)
    // .bind(&buf)
    // .execute(&ctx.db.pool)
    // .await?;

    // Ok(())
}

async fn submit(ctx: &WorkerContext, values: Vec<(Uuid, Vec<u8>)>) -> anyhow::Result<()> {
    let hashes = values.iter().map(|x| x.0).collect::<Vec<_>>();
    let datas = values.into_iter().map(|x| x.1).collect::<Vec<_>>();
    sqlx::query(
        "insert into objects_compressed_cbor_br_9_intkeys (hash, data) select unnest($1), unnest($2) on conflict do nothing",
    )
    .bind(&hashes)
    .bind(&datas)
    .execute(&ctx.db.pool)
    .await?;
    Ok(())
}

fn encode(value: serde_json::Value, key_map: &DashMap<String, u32>, next: &Mutex<u32>) -> Vec<u8> {
    let cur = Vec::new();
    // let mut cbor_data = Vec::new();
    let mut cbor_value = ciborium::Value::serialized(&value).unwrap();
    crunch_keys(&mut cbor_value, key_map, next);

    // let params = BrotliEncoderParams::default();
    let mut writer = CompressorWriter::new(cur, 4096, 9, 21);
    // let str_json = serde_json::to_vec(&value).unwrap();
    // let params = brotli::Br
    // let benc = brotli::BrotliCompress(Cursor::new(str_json), cur, BrotliEncoderParams::)
    // let mut zenc = zstd::Encoder::new(cur, 3).unwrap();
    // serde_json::to_writer(&mut writer, &value).unwrap();
    ciborium::into_writer(&cbor_value, &mut writer).unwrap();
    writer.flush().unwrap();
    writer.into_inner()
    // zenc.finish().unwrap()
    // ciborium::
}

fn crunch_keys(data: &mut ciborium::Value, key_map: &DashMap<String, u32>, next: &Mutex<u32>) {
    match data {
        ciborium::Value::Map(map) => {
            for (key, val) in map {
                if let Some(key_str) = key.as_text_mut() {
                    let key_id = if let Some(key_id) = key_map.get(key_str.as_str()) {
                        *key_id
                    } else {
                        *key_map.entry(key_str.clone()).or_insert_with(|| {
                            let mut mutex = next.lock().expect("shouldn't be poisoned");
                            *mutex += 1;
                            *mutex
                        })
                    };

                    *key = ciborium::Value::Integer(key_id.into());

                    crunch_keys(val, key_map, next);
                }
            }
        }

        ciborium::Value::Array(values) => {
            for val in values {
                crunch_keys(val, key_map, next);
            }
        }
        _ => {}
    }
}
