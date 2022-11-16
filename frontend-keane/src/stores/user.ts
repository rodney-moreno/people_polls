import { defineStore } from "pinia";

export const useUser = defineStore("user", {
  state: () =>
    ({
      email: undefined,
      name: undefined,
    } as { email: string | undefined; name: string | undefined }),
});
