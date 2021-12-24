import Box from "@material-ui/core/Box";
import TextField from "@material-ui/core/TextField";

import { useEditorStyles } from "./editorCommon";
import OutputStatusLine from "./OutputStatusLine";

export default function OutputEditor({ output }: { output: any }) {
  const classes = useEditorStyles();

  return (
    <Box position="relative">
      {/* for Fab positioning */}
      {/* TODO: nowrap */}
      <TextField
        id="input"
        label="Output / 結果"
        placeholder="No input"
        multiline
        fullWidth
        rows={16}
        inputProps={{ wrap: "soft" }}
        value={output ?? ""}
      />
      <Box className={classes.statusLineWrapper}>
        <OutputStatusLine output={output} />
      </Box>
      {/* <WarningFab invalidLines={output?.invalid} /> */}
    </Box>
  );
}
