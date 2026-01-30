import { useMemo } from "react";
import TextField from "@mui/material/TextField";
// import Paper from "@mui/material/Paper";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
import Tooltip from "@mui/material/Tooltip";
import { useApp } from "../AppContext";

import { countLines } from "../utils";

const INPUT_STATS_MAX_LEN: number = 128 * 1024;

export default function InputEditor({
  input,
  setInput,
}: {
  input: string;
  setInput: (value: string) => void;
}) {
  const { wasm } = useApp();
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
        variant="standard"
        sx={{
          "& .MuiInputBase-input": {
            lineHeight: 1.2,
          },
        }}
        value={input}
        onChange={(event) => setInput(event.target.value)}
      />
      <Box sx={{ mt: 0.5, mb: -1 }}>
        <Typography variant="caption" color="textSecondary">
          Lines/橫行:{" "}
          {useMemo(
            () =>
              input.length > INPUT_STATS_MAX_LEN ? (
                <NAwithTooltip />
              ) : (
                countLines(input)
              ),
            [input],
          )}
          <Box
            component="span"
            sx={{ marginLeft: "0.3em", marginRight: "0.3em" }}
          >
            ・
          </Box>
          Chars/字: {input.length}
          <Box
            component="span"
            sx={{ marginLeft: "0.3em", marginRight: "0.3em" }}
          >
            ・
          </Box>
          Variant/變體:{" "}
          {useMemo(() => {
            if (input.length > INPUT_STATS_MAX_LEN) {
              return <NAwithTooltip />;
            }
            if (!wasm) {
              return "LOADING";
            }
            return wasm.infer_variant_confidence(input ?? "");
          }, [input, wasm])}
        </Typography>
      </Box>
    </>
  );
}

function NAwithTooltip() {
  return (
    <Tooltip title="Stats disabled since input size is too large">
      <Box component="span">N/A</Box>
    </Tooltip>
  );
}
