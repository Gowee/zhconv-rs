import React, { useState, useRef, useEffect, useCallback } from "react";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import Paper from "@mui/material/Paper";
import Box from "@mui/material/Box";
import { ThemeProvider } from "@mui/material/styles";
import Backdrop from "@mui/material/Backdrop";
import Typography from "@mui/material/Typography";
import Fab from "@mui/material/Fab";
import Tooltip from "@mui/material/Tooltip";
import FileUploadIcon from "@mui/icons-material/FileUpload";
import { useSnackbar } from "notistack";

import Header from "./components/Header";
import Footer from "./components/Footer";
import InputEditor from "./components/InputEditor";
import OutputEditor from "./components/OutputEditor";
import OptionsControl from "./components/OptionsControl";
import { variants, Variant } from "./components/ConvertButton";
import theme from "./theme";
import { OptionsControlHandle } from "./components/OptionsControl";
import { useApp } from "./AppContext";

import PACKAGE from "../package.json";

function App() {
  const { enqueueSnackbar } = useSnackbar();

  const controlRef = useRef<OptionsControlHandle>(null);

  const [input, setInput] = useState(
    () => localStorage.getItem(`${PACKAGE.name}-text`) || "",
  );

  const [output, setOutput] = useState<string | undefined>(undefined);

  const [dragging, setDragging] = useState(false);

  const { wasm, cgroups } = useApp();

  const [activatedCGroups, setActivatedCGroups] = useState(() => {
    return JSON.parse(
      localStorage.getItem(`${PACKAGE.name}-activated-cgroups`) || "[]",
    ) as string[];
  });

  const [wikitextSupport, setWikitextSupport] = useState(() => {
    return JSON.parse(
      localStorage.getItem(`${PACKAGE.name}-wikitext-support`) || "false",
    ) as boolean;
  });

  const [targetVariant, setTargetVariant] = useState<Variant>(() => {
    const hash = window.location.hash.slice(1) as Variant;

    if (variants[hash]) {
      return hash;
    } else {
      return (
        (localStorage.getItem(`${PACKAGE.name}-target-variant`) as Variant) ??
        "zh"
      );
    }
  });

  const isMounting = useRef(true);

  useEffect(() => {
    if (isMounting.current) {
      isMounting.current = false;
      return;
    }

    const s = JSON.stringify(activatedCGroups);

    localStorage.setItem(`${PACKAGE.name}-activated-cgroups`, s);
  }, [activatedCGroups]);

  const convertText = useCallback(
    async (text: string) => {
      if (!wasm || !cgroups) {
        return;
      }

      const conversionLabel = `conversion (text.len=${text.length}, variant=${targetVariant}, wikitext=${wikitextSupport}, cgroups.len=${activatedCGroups.length})`;
      console.time(conversionLabel);
      const result = await wasm.zhconv(
        text,
        targetVariant,
        wikitextSupport,
        activatedCGroups.map((name) => cgroups.data[name]).join("\n"),
      );
      console.timeEnd(conversionLabel);
      return result;
    },

    [wasm, targetVariant, wikitextSupport, activatedCGroups, cgroups],
  );

  const handleConvert = useCallback(async () => {
    if (input.trim() === "" || !wasm || !cgroups) {
      return;
    }

    setOutput(await convertText(input));
    if (controlRef.current?.controlElement) {
      controlRef.current.controlElement.scrollIntoView({ behavior: "smooth" });
    }
  }, [input, wasm, convertText, cgroups]);

  useEffect(() => {
    if (isMounting.current) {
      return; // Already handled by the first useEffect
    }
    controlRef.current?.clickConvert();
    localStorage.setItem(`${PACKAGE.name}-target-variant`, targetVariant);
    window.history.replaceState({}, "", `#${targetVariant}`);
  }, [targetVariant, controlRef]);

  useEffect(() => {
    if (isMounting.current) {
      return; // Already handled by the first useEffect
    }

    controlRef.current?.clickConvert();
    const s = JSON.stringify(wikitextSupport);
    localStorage.setItem(`${PACKAGE.name}-wikitext-support`, s);
  }, [wikitextSupport, controlRef]);

  const handleFiles = useCallback(
    async (files: File[]) => {
      if (!files.length || !wasm || !cgroups) {
        return;
      }

      const decoder = new TextDecoder("utf-8", { fatal: true });

      for (const file of files) {
        try {
          const buffer = await file.arrayBuffer();
          const text = decoder.decode(buffer);
          if (text.trim() === "") {
            continue;
          }
          const converted = await convertText(text);
          const blob = new Blob([converted ?? ""], { type: "text/plain" });
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;

          let origName = file.name;
          let origExt = "";
          const lastDotIndex = file.name.lastIndexOf(".");

          if (lastDotIndex > 0) {
            origName = file.name.substring(0, lastDotIndex);
            origExt = file.name.substring(lastDotIndex + 1);
          }

          const newFileName = `${origName} ${targetVariant}${origExt ? `.${origExt}` : ""}`;
          a.download = newFileName;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          URL.revokeObjectURL(url);

          enqueueSnackbar(`Converted ${file.name}.`, {
            variant: "success",
          });
        } catch (e: unknown) {
          let reason = "Unknown error";

          if (e instanceof TypeError && e.message.includes("decode")) {
            reason = "Not in valid UTF-8";
          } else if (e instanceof Error) {
            reason = e.message;
          }

          enqueueSnackbar(`Failed to convert ${file.name} (${reason}).`, {
            variant: "error",
          });
        }
      }
    },

    [wasm, convertText, targetVariant, enqueueSnackbar, cgroups],
  );

  const onFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      handleFiles(Array.from(e.target.files));

      e.target.value = "";
    }
  };

  const handleDragEnter = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      setDragging(false);
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setDragging(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      handleFiles(Array.from(e.dataTransfer.files));
      e.dataTransfer.clearData();
    }
  };

  useEffect(() => {
    if (input) {
      localStorage.setItem(`${PACKAGE.name}-text`, input);
    }
  }, [input]);

  return (
    <ThemeProvider theme={theme}>
      <Container
        component="main"
        maxWidth="md"
        sx={{ mt: 5, mb: 2 }}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        <CssBaseline />

        <Header />

        <Box component="main" sx={{ mt: 1, mb: 1 }}>
          <Paper component="section" elevation={3} sx={{ my: 1 }}>
            <Box p={2}>
              <InputEditor input={input} setInput={setInput} />
            </Box>
          </Paper>

          <Paper component="section" elevation={1} sx={{ my: 1 }}>
            <Box p={1}>
              <OptionsControl
                cgroups={cgroups ? Object.keys(cgroups.data) : null}
                activatedCGroups={activatedCGroups}
                onSelectCGroups={setActivatedCGroups}
                wikitextSupport={wikitextSupport}
                onToggleWikitextSupport={() =>
                  setWikitextSupport(!wikitextSupport)
                }
                onConvert={handleConvert}
                targetVariant={targetVariant}
                setTargetVariant={setTargetVariant}
                ref={controlRef}
              />
            </Box>
          </Paper>

          <Paper component="section" elevation={3} sx={{ my: 1 }}>
            <Box p={2}>
              <OutputEditor output={output} />
            </Box>
          </Paper>
        </Box>

        <Footer />

        <Backdrop
          sx={{
            color: "#fff",

            zIndex: (theme) => theme.zIndex.drawer + 1,

            backdropFilter: "blur(3px)",

            backgroundColor: "rgba(0, 0, 0, 0.5)",
          }}
          open={dragging}
        >
          <Typography variant="h4">Drop file to convert</Typography>
        </Backdrop>
      </Container>
      <input
        accept="text/*"
        style={{ display: "none" }}
        id="fab-button-file"
        multiple
        type="file"
        onChange={onFileChange}
      />
      <label htmlFor="fab-button-file">
        <Tooltip
          title={
            <>
              Convert one or more files in UTF-8 encoding
              <br />/ 轉換一個或多個檔案（須為 UTF-8 編碼）
            </>
          }
        >
          <Fab
            sx={{
              position: "fixed",
              bottom: "2rem",
              right: "2rem",
              opacity: 0.5,
              "&:hover": {
                opacity: 1,
              },
            }}
            color="secondary"
            component="span"
          >
            <FileUploadIcon />
          </Fab>
        </Tooltip>
      </label>
    </ThemeProvider>
  );
}

export default App;
