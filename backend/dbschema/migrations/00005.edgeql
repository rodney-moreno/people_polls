CREATE MIGRATION m1aj5tyjg46gbgnjb4c2fed3epom2unebnjjcbxitemewzudtij7ma
    ONTO m1tii2ojoy5znaaugt4q6evxsevgdmuoeabjlvidw7lknkwaezxwxq
{
  ALTER TYPE default::Poll {
      CREATE REQUIRED PROPERTY created_at -> std::datetime {
          SET REQUIRED USING (<std::datetime>'1970-01-01T00:00:00.000Z');
      };
  };
};
