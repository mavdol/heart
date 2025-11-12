import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Loader2, TrashIcon } from "lucide-react";
import { useState } from "react";

export function ConfirmMemoryEraseDialog({
  onDestroyBrain,
}: {
  onDestroyBrain: () => void;
}) {
  const [isDestroying, setIsDestroying] = useState(false);

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <DropdownMenuItem
          className="cursor-pointer"
          onSelect={(e) => e.preventDefault()}
          variant="destructive"
        >
          <div className="flex items-center gap-2 cursor-pointer ">
            <TrashIcon className="text-foreground text-red-800 size-3" />
            Erase heart memories
          </div>
        </DropdownMenuItem>
      </AlertDialogTrigger>
      <AlertDialogContent className="border-border">
        <AlertDialogHeader>
          <AlertDialogTitle className="text-foreground">
            Erase heart memories?
          </AlertDialogTitle>
          <AlertDialogDescription className="text-foreground">
            This action cannot be undone. This will permanently delete all your
            heart memories, souvenirs and reset the conversation history.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel
            className="text-foreground"
            disabled={isDestroying}
          >
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            disabled={isDestroying}
            onClick={(e) => {
              e.preventDefault();
              setIsDestroying(true);

              onDestroyBrain();
            }}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {isDestroying && <Loader2 className="animate-spin" />}
            {isDestroying ? "Erasing..." : "Erase memories"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
