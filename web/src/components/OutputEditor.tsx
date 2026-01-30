import Box from "@mui/material/Box";
import TextField from "@mui/material/TextField";

import OutputStatusLine from "./OutputStatusLine";

export default function OutputEditor({
  output,
}: {
  output: string | undefined;
}) {
  return (
    <Box position="relative">
      {/* for Fab positioning */}
      {/* TODO: nowrap */}
      <TextField
        id="output"
        label="Output / 結果"
        placeholder="No input"
        multiline
        fullWidth
        rows={16}
        variant="standard"
        sx={{
          "& .MuiInputBase-input": {
            lineHeight: 1.2,
          },
        }}
        value={output ?? ""}
      />
      <Box sx={{ mt: 0.5, mb: -1 }}>
        <OutputStatusLine output={output} />
      </Box>
      {/* <WarningFab invalidLines={output?.invalid} /> */}
    </Box>
  );
}
