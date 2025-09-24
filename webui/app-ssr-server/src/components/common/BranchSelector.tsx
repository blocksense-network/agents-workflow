import {
  Component,
  createSignal,
  createResource,
  Show,
  For,
  onMount,
  createEffect,
} from "solid-js";

interface BranchSelectorProps {
  repository: string; // Repository name like "user/repo"
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  class?: string;
}

interface Branch {
  name: string;
  isDefault?: boolean;
}

export const BranchSelector: Component<BranchSelectorProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [searchTerm, setSearchTerm] = createSignal("");

  // Mock branch data - in real implementation this would come from API
  const [branches] = createResource(
    () => props.repository,
    async (repo) => {
      // Mock branches for demo - replace with actual API call
      const mockBranches: Branch[] = [
        { name: "main", isDefault: true },
        { name: "develop" },
        { name: "feature/new-ui" },
        { name: "feature/api-improvements" },
        { name: "bugfix/login-issue" },
        { name: "hotfix/security-patch" },
      ];

      // Filter branches based on search term
      const filtered = mockBranches.filter(branch =>
        branch.name.toLowerCase().includes(searchTerm().toLowerCase())
      );

      return filtered;
    }
  );

  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const value = target.value;
    setSearchTerm(value);
    props.onChange(value);
  };

  const handleBranchSelect = (branch: Branch) => {
    props.onChange(branch.name);
    setSearchTerm("");
    setIsOpen(false);
  };

  const handleInputFocus = () => {
    setIsOpen(true);
    setSearchTerm("");
  };

  const handleInputBlur = () => {
    // Delay closing to allow for click events on options
    setTimeout(() => setIsOpen(false), 150);
  };

  // Close dropdown when clicking outside
  let containerRef: HTMLDivElement | undefined;
  onMount(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef && !containerRef.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  });

  return (
    <div ref={containerRef} class="relative">
      <input
        type="text"
        value={props.value}
        onInput={handleInputChange}
        onFocus={handleInputFocus}
        onBlur={handleInputBlur}
        disabled={props.disabled}
        class={`${props.class} ${
          props.disabled ? "bg-gray-100 cursor-not-allowed" : ""
        }`}
        placeholder="Branch name"
      />

      <Show when={isOpen() && branches()}>
        <div class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-60 overflow-auto">
          <Show
            when={branches()!.length > 0}
            fallback={
              <div class="px-3 py-2 text-sm text-gray-500">
                No branches found
              </div>
            }
          >
            <For each={branches()}>
              {(branch) => (
                <button
                  type="button"
                  onClick={() => handleBranchSelect(branch)}
                  class="w-full px-3 py-2 text-left hover:bg-gray-100 focus:bg-gray-100 focus:outline-none flex items-center justify-between"
                >
                  <span class="text-sm">{branch.name}</span>
                  <Show when={branch.isDefault}>
                    <span class="text-xs text-gray-500 bg-gray-200 px-1 rounded">
                      default
                    </span>
                  </Show>
                </button>
              )}
            </For>
          </Show>
        </div>
      </Show>
    </div>
  );
};
