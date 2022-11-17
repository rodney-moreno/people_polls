<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useUser } from "../stores/user";

const props = defineProps<{ type: "login" | "register" }>();

const router = useRouter();

const email = ref("");
const password = ref("");
const name = ref("");

const error = ref<string | undefined>();

async function handleSubmit(e: Event) {
  e.preventDefault();
  const user = useUser();
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
    const json = await response.json();
    user.email = json.email;
    user.name = json.name;
    if (props.type === "login") {
      router.push("/");
    } else {
      router.push("/login");
    }
  } catch (e: any) {
    error.value = e.message;
  }
}
</script>

<template>
  <form @submit="handleSubmit">
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
    <button class="submitButton">
      <template v-if="type === 'login'">Login</template>
      <template v-else>Register</template>
    </button>
    <p class="errorText" v-if="error">Error: {{ error }}</p>
  </form>
</template>

<style scoped>
.submitButton {
  display: block;
}

.errorText {
  color: red;
}
</style>
