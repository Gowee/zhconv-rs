import React, { useState, useRef, useEffect } from "react";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import Paper from "@mui/material/Paper";
import Box from "@mui/material/Box";
import { ThemeProvider } from "@mui/material/styles";

import Header from "./components/Header";
import Footer from "./components/Footer";
import InputEditor from "./components/InputEditor";
import OutputEditor from "./components/OutputEditor";
import OptionsControl from "./components/OptionsControl";
import theme from "./theme";
import { useWasm } from "./WasmContext";

import PACKAGE from "../package.json";

function App() {
  const controlRef = useRef(null as any);
  const [input, setInput] = useState(
    () => localStorage.getItem(`${PACKAGE.name}-text`) || ""
  );
  const [output, setOutput] = useState(undefined as any);
  const { wasm } = useWasm();

  const handleConvert = async (
    target = "zh",
    mediawiki = false,
    cgroup = ""
  ) => {
    if (input.trim() === "" || !wasm) {
      return;
    }
    setOutput(await wasm.zhconv(input, target, mediawiki, cgroup));
    controlRef?.current &&
      controlRef.current.scrollIntoView({ behavior: "smooth" });
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
      >
        <CssBaseline />
        <Header />
        <Box component="main" sx={{ mt: 1, mb: 1 }}>
          <Paper
            component="section"
            elevation={3}
            sx={{ my: 1 }}
          >
            <Box p={2}>
              <InputEditor input={input} setInput={setInput} />
            </Box>
          </Paper>
          <Paper
            component="section"
            elevation={1}
            sx={{ my: 1 }}
          >
            <Box p={1}>
              <OptionsControl handleConvert={handleConvert} ref={controlRef} />
            </Box>
          </Paper>
          <Paper
            component="section"
            elevation={3}
            sx={{ my: 1 }}
          >
            <Box p={2}>
              <OutputEditor output={output} />
            </Box>
          </Paper>
        </Box>
        <Footer />
      </Container>
    </ThemeProvider>
  );
}

export default App;
