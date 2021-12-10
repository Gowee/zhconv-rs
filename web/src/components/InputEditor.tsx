import { useMemo } from "react";
import TextField from "@material-ui/core/TextField";
// import Paper from "@material-ui/core/Paper";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";

import { countLines } from "../utils";
import { useEditorStyles } from "./editorCommon";

export default function InputEditor({
  input,
  setInput,
}: {
  input: string;
  setInput: (value: string) => void;
}) {
  const classes = useEditorStyles();

  return (
    <>
      <TextField
        id="input"
        label="Input / 原文"
        placeholder="天干物燥，小心火烛。"
        multiline
        fullWidth
        autoFocus
        rows={16}
        inputProps={{ wrap: "soft" }}
        value={input}
        onChange={(event) => setInput(event.target.value)}
      />
      <Box className={classes.statusLineWrapper}>
        <Typography variant="caption" color="textSecondary">
          Lines: {useMemo(() => countLines(input), [input])} / Chars: {input.length}
        </Typography>
      </Box>
    </>
  );
}
