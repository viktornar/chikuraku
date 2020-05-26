-- This file contains the definition of the applications specific roles
-- the roles defined here should not be made owners of database entities (tables/views/...)
\echo # Loading roles
drop role if exists :authenticator;
create role :"authenticator" with login password :'authenticator_pass';
