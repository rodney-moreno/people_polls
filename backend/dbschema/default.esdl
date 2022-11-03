module default {
  
  scalar type Choice extending enum<ChoiceA, ChoiceB>;

  type User {
    required property email -> str {
      constraint exclusive;
      constraint min_len_value(1);
    }

    property name -> str;
    
    multi link poll_responses -> PollResponse {
      constraint exclusive;
    }
  }

  type Poll {
    required property question_text -> str;
    required property prompt_a -> str;
    required property prompt_b -> str;
    multi link poll_responses -> PollResponse {
      constraint exclusive;
    }
  }

  type PollResponse {
    required property choice -> Choice;
    required link user -> User;
    required link poll -> Poll;
  }
}