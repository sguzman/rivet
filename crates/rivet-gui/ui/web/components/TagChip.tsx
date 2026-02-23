import Chip from "@mui/material/Chip";

import { tagColorStyle } from "../lib/tags";
import { useAppStore } from "../store/useAppStore";

interface TagChipProps {
  tag: string;
  size?: "small" | "medium";
}

export function TagChip(props: TagChipProps) {
  const schema = useAppStore((state) => state.tagSchema);
  const colorMap = useAppStore((state) => state.tagColorMap);
  const color = tagColorStyle(props.tag, schema, colorMap);

  return (
    <Chip
      label={props.tag}
      size={props.size ?? "small"}
      sx={{
        borderColor: color,
        color,
        backgroundColor: "color-mix(in srgb, var(--mui-palette-background-paper) 84%, transparent)",
        borderWidth: 1,
        borderStyle: "solid",
        "& .MuiChip-label": {
          fontFamily: "\"Source Code Pro\", monospace",
          fontSize: "0.72rem"
        }
      }}
    />
  );
}
