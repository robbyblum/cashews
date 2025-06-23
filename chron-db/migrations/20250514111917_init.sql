create extension "pgcrypto";

create table observations (
    kind smallint not null,
    entity_id text not null,
    timestamp timestamptz not null,
    request_time float not null,
    hash uuid not null
);
create index observations_idx on observations(kind, entity_id, timestamp desc);

-- used for version "locking", and good for reads too
create table latest_versions (
    kind smallint not null,
    entity_id text not null,
    seq int not null,
    hash uuid not null,
    valid_from timestamptz not null,
    primary key (kind, entity_id)
);

create table versions (
    kind smallint not null,
    seq int not null,
    entity_id text not null,
    hash uuid not null,
    valid_from timestamptz not null,
    valid_to timestamptz default null,
    last_seen timestamptz default null,
    primary key (kind, entity_id, seq)
);

create table objects (
    hash uuid primary key,
    data jsonb not null
);