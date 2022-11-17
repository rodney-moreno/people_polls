import { defineStore } from "pinia";
import { ref } from "vue";

export const useUser = defineStore("user", () => {
  const _isInitialFetchDone = ref(false);
  const _email = ref<string>();
  const _name = ref<string>();

  function login(email: string, name: string) {
    _email.value = email;
    _name.value = name;
  }

  function logout() {
    _email.value = undefined;
    _name.value = undefined;
  }

  return {
    isInitialFetchDone: _isInitialFetchDone,
    email: _email,
    name: _name,
    login,
    logout,
  };
});
