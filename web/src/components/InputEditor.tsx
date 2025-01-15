import { useMemo, useState, useEffect } from "react";
import TextField from "@material-ui/core/TextField";
// import Paper from "@material-ui/core/Paper";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";
import Tooltip from "@material-ui/core/Tooltip";

import { countLines } from "../utils";
import { useEditorStyles } from "./editorCommon";

const INPUT_STATS_MAX_LEN: number = 128 * 1024;

export default function InputEditor({
  input,
  setInput,
}: {
  input: string;
  setInput: (value: string) => void;
}) {
  const classes = useEditorStyles();
  const [inferVariantConfidence, setInferVariantConfidence] = useState(
    () => (_: string) => "LOADING"
  );
  // useEffect(() => {
  //   () => import("../../../pkg/zhconv.js").then(({ infer_variant_confidence }) => setInferVariantConfidence(infer_variant_confidence))
  // }, []);
  useEffect(() => {
    const loadMod = async () => {
      const { infer_variant_confidence } = await import(
        "../../../pkg/zhconv.js"
      );
      setInferVariantConfidence(() => infer_variant_confidence);
    };
    loadMod();
  }, []);
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
          Lines/橫行:{" "}
          {useMemo(
            () =>
              input.length > INPUT_STATS_MAX_LEN ? (
                <NAwithTooltip />
              ) : (
                countLines(input)
              ),
            [input]
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
          {useMemo(
            () =>
              input.length > INPUT_STATS_MAX_LEN ? (
                <NAwithTooltip />
              ) : (
                inferVariantConfidence(input ?? "")
              ),
            [input, inferVariantConfidence]
          )}
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
