<script lang="ts">
  import { profilesStore } from "../stores/profiles";

  const { activeProfile, activeProfileName } = profilesStore;

  function onProfileChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    profilesStore.activeProfileName.set(target.value);
  }

  function addProfile() {
    const name = prompt("输入新方案名称:");
    if (!name) return;
    const current = $activeProfile;
    if (!current) return;
    profilesStore.saveProfile({
      ...current,
      name,
    });
  }

  function renameProfile() {
    const current = $activeProfile;
    if (!current) return;
    const newName = prompt("输入新名称:", current.name);
    if (!newName) return;
    profilesStore.saveProfile({
      ...current,
      name: newName,
    });
    profilesStore.activeProfileName.set(newName);
  }

  function deleteProfile() {
    const current = $activeProfile;
    if (!current) return;
    if (!confirm(`确定要删除方案 "${current.name}" 吗？`)) return;
    profilesStore.deleteProfile(current.name);
  }
</script>

<div class="profile-selector">
  <label>配置方案:</label>
  <div class="profile-row">
    <select onchange={onProfileChange} value={$activeProfileName}>
      {#each $profilesStore as profile}
        <option value={profile.name}>{profile.name}</option>
      {/each}
    </select>
    <button onclick={addProfile} title="新建方案">+</button>
    <button onclick={renameProfile} title="重命名">✎</button>
    <button onclick={deleteProfile} title="删除">✕</button>
  </div>
</div>

<style>
  .profile-selector {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .profile-selector label {
    font-size: 13px;
    white-space: nowrap;
  }
  .profile-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  select {
    padding: 4px 8px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
    background: var(--bg-secondary);
  }
  button {
    padding: 2px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-secondary);
    cursor: pointer;
    font-size: 12px;
  }
  button:hover {
    border-color: var(--accent);
  }
</style>
