<script lang="ts">
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { VaultIcon, FileLockIcon, FileKeyIcon, SettingsIcon, PanelLeft} from "@lucide/svelte";

    const items = [
        { title: "keys", url: "/", icon: VaultIcon },
        { title: "encrypt", url: "/encrypt", icon: FileLockIcon },
        { title: "decrypt", url: "/decrypt", icon: FileKeyIcon },
        // { title: "settings", url: "settings", icon: SettingsIcon }
    ];
    const sidebar = Sidebar.useSidebar();
</script>

<Sidebar.Root variant="floating" collapsible="icon" class="ease-out">
    <Sidebar.Content>
        <Sidebar.Group>
              <Sidebar.GroupLabel>CHIFFRAGE</Sidebar.GroupLabel>
              <Sidebar.GroupContent>
                <Sidebar.Menu>
                    <Sidebar.MenuItem>
                      <Sidebar.MenuButton>
                        {#snippet child({ props })}
                          <a onclick={sidebar.toggle} {...props}>
                            <PanelLeft />
                            <span class="text-md">collapse sidebar</span>
                          </a>
                        {/snippet}
                      </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
                    <Sidebar.Separator/>
                  {#each items as item (item.title)}
                    <Sidebar.MenuItem>
                      <Sidebar.MenuButton>
                        {#snippet child({ props })}
                          <a href={item.url} {...props}>
                            <item.icon />
                            <span class="text-md">{item.title}</span>
                          </a>
                        {/snippet}
                      </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
                  {/each}
                </Sidebar.Menu>
              </Sidebar.GroupContent>
            </Sidebar.Group>
    </Sidebar.Content>
</Sidebar.Root>
