-- Add down migration script here

-- AWS DES support this: https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/CHAP_PostgreSQL.html#PostgreSQL.Concepts.General.Extensions.Trusted
DROP SCHEMA rsvp CASCADE;
