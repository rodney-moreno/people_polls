<script setup lang="ts">
import { useUser } from "@/stores/user";
import { onMounted } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();
const user = useUser();
let error: string | undefined;
onMounted(async () => {
  const response = await fetch("http://127.0.0.1:8080/logout", {
    credentials: "include",
  });
  if (response.status !== 200) {
    error = "Failed to log out. Please try again by refreshing the page.";
    throw new Error(`Unexpected status code: ${response.status}`);
  }
  user.logout();
  await router.push("/");
});
</script>

<template>
  <p v-if="error">{{ error }}</p>
  <p v-else>Logging out...</p>
</template>
