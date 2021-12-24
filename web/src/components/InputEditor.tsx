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
        inputProps={{ wrap: "soft", fontSize: "1.2em" }}
        value={input}
        onChange={(event) => setInput(event.target.value)}
      />
      <Box className={classes.statusLineWrapper}>
        <Typography variant="caption" color="textSecondary">
          Lines/橫行: {useMemo(() => countLines(input), [input])}
          <Box
            component="span"
            sx={{ marginLeft: "0.3em", marginRight: "0.3em" }}
          >
            ・
          </Box>
          Chars/字: {input.length}
        </Typography>
      </Box>
    </>
  );
}
