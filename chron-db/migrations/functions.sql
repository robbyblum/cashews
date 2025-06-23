
create or replace function add_version(new_kind smallint, new_entity_id text, new_hash uuid, new_timestamp timestamptz, new_request_time float)
    returns integer
    language plpgsql as $$
declare
    last_observation record;
    updated_lv record;
begin
    -- this should be atomic
    insert into latest_versions (kind, entity_id, seq, hash, valid_from)
        -- try to insert if this is a new object entirely
        values (new_kind, new_entity_id, 0, new_hash, new_timestamp)
        on conflict (kind, entity_id) do update
            -- if it isn't, try to bump the seq and update the hash
            set seq = latest_versions.seq + 1, hash = new_hash, valid_from = new_timestamp
            -- but only if the hash is different and the timestamp is newer
            where latest_versions.hash != new_hash and latest_versions.valid_from < new_timestamp
        returning latest_versions.seq into updated_lv;

    if found then
        -- we have a new version
        insert into versions (kind, entity_id, seq, hash, valid_from)
            values (new_kind, new_entity_id, updated_lv.seq, new_hash, new_timestamp);

        -- get old last seen
        select timestamp from observations
            where kind = new_kind and entity_id = new_entity_id
            order by timestamp desc limit 1
            into last_observation;

        -- close off the old one
        update versions
            set valid_to = new_timestamp, last_seen = last_observation.timestamp
            where kind = new_kind and entity_id = new_entity_id and seq = updated_lv.seq - 1;
    end if;

    insert into observations (kind, entity_id, timestamp, request_time, hash)
        values (new_kind, new_entity_id, new_timestamp, new_request_time, new_hash);

    return found::integer;
end
$$;


create or replace function rebuild_entity(the_kind smallint, the_entity_id text) returns integer as $$
declare
    row record;             -- temp var
    last_ver_update record; -- last seen UPDATE that's the START of a new VERSION
    last_timestamp timestamptz;  -- last seen timestamp (previous iteration of loop)
    num_versions int;       -- counter
begin
    -- bump latest versions timestamp to "lock" it - so add_version won't clobber it
    -- we could probably do this with a real row lock but idk
    insert into latest_versions (kind, entity_id, seq, hash, valid_from)
        values (the_kind, the_entity_id, 0, '00000000-0000-0000-0000-000000000000', '9999-01-01T00:00:00Z')
        on conflict (kind, entity_id) do update
            set seq = 0, hash = '00000000-0000-0000-0000-000000000000', valid_from = '9999-01-01T00:00:00Z';

    -- clear this entity to start from scratch
    delete from versions v 
        where v.kind = the_kind and v.entity_id = the_entity_id;
    
    num_versions := 0;
    last_ver_update := null;
    last_timestamp := null;
    
    for row in 
        select hash, timestamp from observations u
            where u.kind = the_kind and u.entity_id = the_entity_id
            order by timestamp
    loop
        if last_ver_update is null then
            last_ver_update := row;
        elsif last_ver_update.hash is distinct from row.hash then        
            -- when we detect a new hash, insert the *previous* hash's version
            -- staying one version behind so we have both the start/end (no need for an extra query)
            insert into versions (kind, entity_id, hash, valid_from, valid_to, last_seen, seq) 
                select
                    the_kind, 
                    the_entity_id,
                    last_ver_update.hash, 
                    last_ver_update.timestamp,
                    row.timestamp,
                    last_timestamp,
                    num_versions;
            
            last_ver_update := row;
            num_versions := num_versions + 1;
        end if;

        last_timestamp := row.timestamp;
    end loop;
    
    -- insert the latest version that would've been missed by the above loop
    if last_ver_update is not null then
        insert into versions (kind, entity_id, hash, valid_from, valid_to, last_seen, seq) 
            select
                the_kind,
                the_entity_id,
                last_ver_update.hash,
                last_ver_update.timestamp,
                null,
                null,
                num_versions;

        insert into latest_versions (kind, entity_id, seq, hash, valid_from)
            values (the_kind, the_entity_id, num_versions, last_ver_update.hash, last_ver_update.timestamp)
            on conflict (kind, entity_id) do update
                set seq = num_versions, hash = last_ver_update.hash, valid_from = last_ver_update.timestamp;

        num_versions := num_versions + 1;
    else
        -- we didn't find any updates at all, remove the "lock"
        delete from latest_versions where kind = the_kind and entity_id = the_entity_id;
    end if;

    return num_versions;
end;
$$ language plpgsql;
