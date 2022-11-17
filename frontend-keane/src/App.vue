<script setup lang="ts">
import ProgressBar from "@/components/ProgressBar.vue";
import { useUser } from "@/stores/user";
import useSWRV from "swrv";
import { RouterLink, RouterView, useRouter } from "vue-router";

const router = useRouter();
const user = useUser();

const { data, error } = useSWRV("http://127.0.0.1:8080/me", async (key) => {
  const response = await fetch(key, { credentials: "include" });
  user.isInitialFetchDone = true;
  if (response.status !== 200) {
    // We are logged out. Update local state to reflect this.
    user.logout();
    await router.push("/login");
    throw new Error(`Unexpected status code: ${response.status}`);
  }
  const responseData = await response.json();
  user.login(responseData.email, responseData.name);
  return responseData;
});
</script>

<template>
  <header>
    <nav>
      <RouterLink to="/">Home</RouterLink>
      <RouterLink to="/about">About</RouterLink>
      <RouterLink v-if="user.email" to="/logout">Logout</RouterLink>
    </nav>
  </header>

  <RouterView v-if="data || error" />
  <ProgressBar v-else />
</template>

<style scoped></style>
