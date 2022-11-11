CREATE MIGRATION m1tii2ojoy5znaaugt4q6evxsevgdmuoeabjlvidw7lknkwaezxwxq
    ONTO m1cyetoespuaov4s3nzipd6kniw5r46uzp7j3it4ruk7o2n3kkgkkq
{
  ALTER TYPE default::User {
      CREATE REQUIRED PROPERTY password_hash -> std::str {
          SET REQUIRED USING ('REPLACE_ME');
      };
  };
};
