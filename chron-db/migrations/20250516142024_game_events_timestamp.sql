alter table game_events add column observed_at timestamptz;
create index game_events_observed_at_idx on game_events(observed_at desc);
