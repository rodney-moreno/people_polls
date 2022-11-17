<script setup lang="ts">
import { useUser } from "@/stores/user";
import { ref } from "vue";
import { useRouter } from "vue-router";

const props = defineProps<{ type: "login" | "register" }>();

const router = useRouter();
const user = useUser();

const email = ref("");
const password = ref("");
const name = ref("");
const isLoginLoading = ref(false);

const error = ref<string | undefined>();

async function handleSubmit(e: Event) {
  e.preventDefault();
  isLoginLoading.value = true;
  try {
    const response = await fetch(`http://127.0.0.1:8080/${props.type}`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        email: email.value,
        password: password.value,
        ...(props.type === "register" && { name: name.value }),
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
    if (props.type === "login") {
      const json = await response.json();
      user.login(json.email, json.name);
      router.push("/");
    } else {
      router.push("/login");
    }
  } catch (e: any) {
    error.value = e.message;
    throw e;
  } finally {
    isLoginLoading.value = false;
  }
}
</script>

<template>
  <form v-if="!user.email" @submit="handleSubmit">
    <label>
      Email
      <input type="text" placeholder="email@provider.com" v-model="email" />
    </label>
    <label>
      Password
      <input type="password" v-model="password" />
    </label>
    <label v-if="type === 'register'">
      Name
      <input type="text" placeholder="Bob" v-model="name" />
    </label>
    <button class="submitButton" :disabled="isLoginLoading">
      <template v-if="isLoginLoading">Loading...</template>
      <template v-else-if="type === 'login'">Login</template>
      <template v-else>Register</template>
    </button>
    <p class="errorText" v-if="error">Error: {{ error }}</p>
  </form>
  <p v-else>Already logged in.</p>
</template>

<style scoped>
.submitButton {
  display: block;
}

.errorText {
  color: red;
}
</style>
