CREATE MIGRATION m1cyetoespuaov4s3nzipd6kniw5r46uzp7j3it4ruk7o2n3kkgkkq
    ONTO m1eqcixb5537ng5fmbdcjo3t55f7xlbc6mbp4ipuji2snzbz7c7kxa
{
  ALTER TYPE default::PollResponse {
      CREATE CONSTRAINT std::exclusive ON ((.user, .poll));
  };
};
