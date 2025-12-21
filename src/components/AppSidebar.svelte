<script lang="ts">
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import HelpDialog from "./HelpDialog.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import { VaultIcon, FileLockIcon, FileKeyIcon, PanelLeft, CircleQuestionMarkIcon } from "@lucide/svelte";
    import {page} from "$app/state"

    const items = [
        { title: "keys", url: "/", icon: VaultIcon },
        { title: "encrypt", url: "/encrypt", icon: FileLockIcon },
        { title: "decrypt", url: "/decrypt", icon: FileKeyIcon },
        // { title: "settings", url: "settings", icon: SettingsIcon }
    ];
    const sidebar = Sidebar.useSidebar();
    let helpOpen = $state(false);
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
                      <Sidebar.MenuButton isActive={new URL(page.url).pathname === item.url}>
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
    <Sidebar.Footer>
        <Button class="w-full" variant="ghost" onclick={() => helpOpen = true}>
            <CircleQuestionMarkIcon />
        </Button>
    </Sidebar.Footer>
</Sidebar.Root>
<HelpDialog bind:open={helpOpen} />
