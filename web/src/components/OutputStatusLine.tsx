import { useMemo } from "react";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";

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
