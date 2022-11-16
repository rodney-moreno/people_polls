<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useUser } from "../stores/user";

const router = useRouter();

const email = ref("");
const password = ref("");
const error = ref<string | undefined>();

async function handleSubmit(e: Event) {
  e.preventDefault();
  const user = useUser();
  try {
    const response = await fetch("http://localhost:8080/login", {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        email: email.value,
        password: password.value,
      }),
      credentials: "include",
    });
    if (response.status >= 400) {
      try {
        const reason = await response.text();
        error.value = reason;
      } catch (e) {
        error.value = "An unexpected error occurred.";
        throw e;
      }
      throw new Error(error.value);
    }
    const json = await response.json();
    user.email = json.email;
    user.name = json.name;
    router.push("/");
  } catch (e: any) {
    error.value = e.message;
  }
}
</script>

<template>
  <h1>Login</h1>
  <form @submit="handleSubmit">
    <label>
      Email
      <input type="text" placeholder="email@provider.com" v-model="email" />
    </label>
    <label>
      Password
      <input type="password" v-model="password" />
    </label>
    <button class="loginButton">Login</button>
    <p class="errorText" v-if="error">Error: {{ error }}</p>
  </form>
</template>

<style scoped>
.loginButton {
  display: block;
}

.errorText {
  color: red;
}
</style>
