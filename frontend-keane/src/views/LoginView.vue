<script setup lang="ts">
import LoginRegister from "@/components/LoginRegister.vue";
import { RouterLink } from "vue-router";
</script>

<template>
  <h1>Login</h1>
  <LoginRegister type="login" />
  <RouterLink to="/register">Register</RouterLink>
</template>

<style scoped></style>
