import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Profile } from "../types";

function createProfilesStore() {
  const store = writable<Profile[]>([]);
  const { subscribe, set, update } = store;
  const activeProfileName = writable<string>("常用");
  const loading = writable<boolean>(false);

  // Capture activeProfileName for closures
  let $activeProfileName = "常用";
  activeProfileName.subscribe((v) => ($activeProfileName = v));

  const activeProfile = derived(
    [store, activeProfileName],
    ([$profiles, $name]: [Profile[], string]) =>
      $profiles.find((p) => p.name === $name) ?? $profiles[0] ?? null
  );

  async function loadProfiles() {
    loading.set(true);
    try {
      const profiles = await invoke<Profile[]>("get_profiles");
      set(profiles);
      if (profiles.length > 0 && !profiles.find((p) => p.name === $activeProfileName)) {
        activeProfileName.set(profiles[0].name);
      }
    } catch (e) {
      console.error("Failed to load profiles:", e);
    } finally {
      loading.set(false);
    }
  }

  async function saveProfile(profile: Profile) {
    await invoke("save_profile", { profile });
    await loadProfiles();
  }

  async function deleteProfile(name: string) {
    await invoke("delete_profile", { name });
    await loadProfiles();
  }

  function updateActiveProfile(partial: Partial<Profile>) {
    update((profiles) =>
      profiles.map((p) =>
        p.name === $activeProfileName ? { ...p, ...partial } : p
      )
    );
  }

  return {
    subscribe,
    activeProfile,
    activeProfileName,
    loading,
    loadProfiles,
    saveProfile,
    deleteProfile,
    updateActiveProfile,
  };
}

export const profilesStore = createProfilesStore();
