CREATE MIGRATION m1eqcixb5537ng5fmbdcjo3t55f7xlbc6mbp4ipuji2snzbz7c7kxa
    ONTO m1dfen2yketpgkcg7mja3frpeni7nszif7dbfhqqfe4an6eczrl34q
{
  ALTER TYPE default::Poll {
      CREATE REQUIRED PROPERTY is_approved -> std::bool {
          SET REQUIRED USING (false);
      };
  };
};
