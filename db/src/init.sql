-- some setting to make the output less verbose
\set QUIET on
\set ON_ERROR_STOP on
\set client_min_messages to warning;

-- load some variables from the env
\setenv base_dir :DIR
\set base_dir `if [ $base_dir != ":"DIR ]; then echo $base_dir; else echo "/docker-entrypoint-initdb.d"; fi`
\set authenticator `echo $DB_USER`
\set authenticator_pass `echo $DB_PASS`

\echo # Loading database definition
begin;
create extension if not exists pgcrypto;

-- functions for sending messages to RabbitMQ entities
\ir libs/rabbitmq.sql
-- user with role creation
\ir authorization/roles.sql

commit;
\echo # ==========================================
