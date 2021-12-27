import React, { useState, useRef, useEffect } from "react";
import CssBaseline from "@material-ui/core/CssBaseline";
import { makeStyles } from "@material-ui/core/styles";
import Container from "@material-ui/core/Container";
import Paper from "@material-ui/core/Paper";
import Box from "@material-ui/core/Box";

import Header from "./components/Header";
import Footer from "./components/Footer";
import InputEditor from "./components/InputEditor";
import OutputEditor from "./components/OutputEditor";
import OptionsControl from "./components/OptionsControl";

import PACKAGE from "../package.json";

(async () => {
  // preload wasm
  await import("../../pkg/zhconv.js");
})();

const useStyles = makeStyles((theme) => ({
  // root: {
  //   display: 'flex',
  //   flexDirection: 'column',
  //   minHeight: '100vh',
  // },
  main: {
    marginTop: theme.spacing(5),
    marginBottom: theme.spacing(2),
  },
  editorWrapper: {
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(2),
    padding: theme.spacing(2),
  },
  optionsControlWrapper: {
    marginTop: theme.spacing(-1),
    marginBottom: theme.spacing(-1),
    padding: theme.spacing(1),
    // '& > *': {
    //   margin: theme.spacing(1)
    // }
  },
  mainWrapper: {
    marginTop: theme.spacing(1),
    marginBottom: theme.spacing(1),
  },
  sectionWrapper: {
    marginTop: theme.spacing(1),
    marginBottom: theme.spacing(1),
  },
  // footer: {
  //   padding: theme.spacing(3, 2),
  //   marginTop: 'auto',
  //   backgroundColor:
  //     theme.palette.type === 'light' ? theme.palette.grey[200] : theme.palette.grey[800],
  // },
}));

function App() {
  const classes = useStyles();
  const controlRef = useRef(null as any);
  const [input, setInput] = useState(
    () => localStorage.getItem(`${PACKAGE.name}-text`) || ""
  );
  const [output, setOutput] = useState(undefined as any);
  const handleConvert = async (
    target = "zh",
    mediawiki = false,
    cgroup = ""
  ) => {
    if (input.trim() === "") {
      return;
    }
    const { zhconv } = await import("../../pkg/zhconv.js");
    setOutput(await zhconv(input, target, mediawiki, cgroup));
    controlRef?.current &&
      controlRef.current.scrollIntoView({ behavior: "smooth" });
  };
  useEffect(() => {
    if (input) {
      localStorage.setItem(`${PACKAGE.name}-text`, input);
    }
  }, [input]);

  return (
    <Container component="main" className={classes.main} maxWidth="md">
      <CssBaseline />
      <Header />
      <main className={classes.mainWrapper}>
        <Paper
          component="section"
          elevation={3}
          className={classes.sectionWrapper}
        >
          <Box p={2}>
            <InputEditor input={input} setInput={setInput} />
          </Box>
        </Paper>
        <Paper
          component="section"
          elevation={1}
          className={classes.sectionWrapper}
        >
          <Box p={1}>
            <OptionsControl handleConvert={handleConvert} ref={controlRef} />
          </Box>
        </Paper>
        <Paper
          component="section"
          elevation={3}
          className={classes.sectionWrapper}
        >
          <Box p={2}>
            <OutputEditor output={output} />
          </Box>
        </Paper>
      </main>
      <Footer />
    </Container>
  );
}

export default App;
