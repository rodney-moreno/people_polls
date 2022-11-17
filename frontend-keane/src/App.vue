<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
import useSWRV from "swrv";
import { useUser } from "./stores/user";

useSWRV("http://127.0.0.1:8080/me", async (key) => {
  const response = await fetch(key, { credentials: "include" });
  if (response.status !== 200) {
    throw new Error(`Unexpected status code: ${response.status}`);
  }
  const responseData = await response.json();
  const user = useUser();
  user.email = responseData.email;
  user.name = responseData.name;
});
</script>

<template>
  <header>
    <nav>
      <RouterLink to="/">Home</RouterLink>
      <RouterLink to="/about">About</RouterLink>
    </nav>
  </header>

  <RouterView />
</template>

<style scoped></style>
