import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SettingsIcon } from "lucide-react";
import { ConfirmMemoryEraseDialog } from "./confirmMemoryEraseDialog";

export function SettingsDropdown({
  onDestroyBrain,
}: {
  onDestroyBrain: () => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <div className="p-.5 hover:bg-accent rounded-sm group cursor-pointer">
          <SettingsIcon className="text-muted-foreground group-hover:text-foreground  size-4" />
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-auto m-3 border-border" align="start">
        <DropdownMenuGroup>
          <ConfirmMemoryEraseDialog onDestroyBrain={onDestroyBrain} />
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
