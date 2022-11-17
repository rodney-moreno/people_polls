<script setup lang="ts">
import useSWRV from "swrv";

const { data, error } = useSWRV(
  "http://127.0.0.1:8080/polls?hasVotedIn=false",
  async (key) => {
    return await fetch(key, { credentials: "include" }).then((response) =>
      response.json()
    );
  }
);
</script>

<template>
  <h2>Polls you haven't voted in yet</h2>
  <div v-if="error">failed to load</div>
  <div v-if="!data">loading...</div>
  <div v-else>{{ JSON.stringify(data) }}</div>
</template>
