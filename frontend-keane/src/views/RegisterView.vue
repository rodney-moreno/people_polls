<script setup lang="ts">
import LoginRegister from "@/components/LoginRegister.vue";
import { RouterLink } from "vue-router";
</script>

<template>
  <h1>Register</h1>
  <LoginRegister type="register" />
  <RouterLink to="/login">Login</RouterLink>
</template>

<style scoped></style>
