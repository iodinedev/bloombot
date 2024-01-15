CREATE INDEX ON term (aliases);
CREATE INDEX ON term USING GIN (aliases);