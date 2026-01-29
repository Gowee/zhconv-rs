import { useMemo } from "react";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";

import { countLines } from "../utils";

export default function OutputStatusLine({ output }: { output: any }) {
  return (
    <Typography variant="caption" color="textSecondary">
      Lines/橫行: {useMemo(() => countLines(output), [output])}
      <Box component="span" sx={{ marginLeft: "0.3em", marginRight: "0.3em" }}>
        ・
      </Box>
      Chars/字: {output ? output.length : 0}
    </Typography>
  );
}
