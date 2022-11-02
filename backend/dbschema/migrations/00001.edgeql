CREATE MIGRATION m1dfen2yketpgkcg7mja3frpeni7nszif7dbfhqqfe4an6eczrl34q
    ONTO initial
{
  CREATE TYPE default::Poll {
      CREATE REQUIRED PROPERTY prompt_a -> std::str;
      CREATE REQUIRED PROPERTY prompt_b -> std::str;
      CREATE REQUIRED PROPERTY question_text -> std::str;
  };
  CREATE SCALAR TYPE default::Choice EXTENDING enum<ChoiceA, ChoiceB>;
  CREATE TYPE default::PollResponse {
      CREATE REQUIRED LINK poll -> default::Poll;
      CREATE REQUIRED PROPERTY choice -> default::Choice;
  };
  ALTER TYPE default::Poll {
      CREATE MULTI LINK poll_responses -> default::PollResponse {
          CREATE CONSTRAINT std::exclusive;
      };
  };
  CREATE TYPE default::User {
      CREATE MULTI LINK poll_responses -> default::PollResponse {
          CREATE CONSTRAINT std::exclusive;
      };
      CREATE REQUIRED PROPERTY email -> std::str {
          CREATE CONSTRAINT std::exclusive;
          CREATE CONSTRAINT std::min_len_value(1);
      };
      CREATE PROPERTY name -> std::str;
  };
  ALTER TYPE default::PollResponse {
      CREATE REQUIRED LINK user -> default::User;
  };
};
