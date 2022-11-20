<script setup lang="ts">
import useSWRV from "swrv";
import { reactive, ref } from "vue";

type Tab = "voted" | "not-voted";

const {
  data: hasNotVotedInData,
  error: hasNotVotedInError,
  mutate: hasNotVotedInMutate,
} = useSWRV("/polls?hasVotedIn=false", async (key) => {
  return await fetch("http://127.0.0.1:8080" + key, {
    credentials: "include",
  }).then((response) => response.json());
});

const {
  data: hasVotedInData,
  error: hasVotedInError,
  mutate: hasVotedInMutate,
} = useSWRV("/polls?hasVotedIn=true", async (key) => {
  return await fetch("http://127.0.0.1:8080" + key, {
    credentials: "include",
  }).then((response) => response.json());
});

const tab = ref<Tab>("voted");

const choiceState = reactive<Record<string, boolean | undefined>>({});

function handleTabClick(newTab: Tab) {
  tab.value = newTab;
}

async function handleSubmit(pollId: string) {
  const choice = choiceState[pollId];
  if (choice === undefined) {
    return;
  }

  await fetch("http://127.0.0.1:8080/pollResponses", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      poll_id: pollId,
      choice: choice,
    }),
    credentials: "include",
  });
  await Promise.all([hasNotVotedInMutate(), hasVotedInMutate()]);
}
</script>

<template>
  <div>
    <div class="window" style="width: 400px">
      <div class="title-bar">
        <div class="title-bar-text">
          <template v-if="tab === 'not-voted'"
            >Polls you haven't voted in yet
          </template>
          <template v-if="tab === 'voted'">Polls you voted already</template>
        </div>
        <div class="title-bar-controls">
          <button aria-label="Minimize"></button>
          <button aria-label="Maximize"></button>
          <button aria-label="Close"></button>
        </div>
      </div>
      <div class="window-body">
        <menu role="tablist">
          <button
            :aria-selected="tab === 'voted' ? 'true' : 'false'"
            aria-controls="voted"
            @click="handleTabClick('voted')"
          >
            Not Voted
          </button>
          <button
            :aria-selected="tab === 'not-voted' ? 'true' : 'false'"
            aria-controls="not-voted"
            @click="handleTabClick('not-voted')"
          >
            Voted
          </button>
        </menu>
        <article
          role="tabpanel"
          :hidden="tab !== 'voted' ? true : false"
          id="voted"
        >
          <div v-if="hasNotVotedInError">failed to load</div>
          <progress v-if="!hasNotVotedInData"></progress>
          <template
            v-else
            v-for="(poll, index) in hasNotVotedInData"
            :key="poll.id"
          >
            <p>{{ poll.question_text }}</p>
            <fieldset>
              <div class="field-row">
                <input
                  :id="`radio-${index}-a`"
                  type="radio"
                  :name="poll.id"
                  v-model="choiceState[poll.id]"
                  :value="true"
                />
                <label :for="`radio-${index}-a`">{{ poll.prompt_a }}</label>
              </div>
              <div class="field-row">
                <input
                  :id="`radio-${index}-b`"
                  type="radio"
                  :name="poll.id"
                  v-model="choiceState[poll.id]"
                  :value="false"
                />
                <label :for="`radio-${index}-b`">{{ poll.prompt_b }}</label>
              </div>
            </fieldset>
            <section class="field-row">
              <button
                @click="handleSubmit(poll.id)"
                :disabled="choiceState[poll.id] == undefined"
              >
                Submit
              </button>
              <label>{{ poll.id }}</label>
            </section>
          </template>
        </article>

        <article
          role="tabpanel"
          :hidden="tab !== 'not-voted' ? true : false"
          id="not-voted"
        >
          <div v-if="hasVotedInError">failed to load</div>
          <progress v-if="!hasVotedInData"></progress>
          <template
            v-else
            v-for="(poll, index) in hasVotedInData"
            :key="poll.id"
          >
            <p>{{ poll.question_text }}</p>
            <fieldset>
              <div class="field-row">
                <input
                  :id="`radio-${index}-a`"
                  type="radio"
                  :name="poll.id"
                  :checked="poll.user_response.choice === 'ChoiceA'"
                  disabled
                />
                <label :for="`radio-${index}-a`">{{ poll.prompt_a }}</label>
              </div>
              <div class="field-row">
                <input
                  :id="`radio-${index}-b`"
                  type="radio"
                  :name="poll.id"
                  :checked="poll.user_response.choice === 'ChoiceB'"
                  disabled
                />
                <label :for="`radio-${index}-b`">{{ poll.prompt_b }}</label>
              </div>
            </fieldset>
          </template>
        </article>

        <section class="field-row" style="justify-content: flex-end">
          <button>OK</button>
          <button>Cancel</button>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import "xp.css/dist/XP.css";
</style>
