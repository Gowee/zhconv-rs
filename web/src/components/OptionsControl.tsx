import { forwardRef, ForwardedRef, useState, useEffect, useRef } from "react";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";
import Grid from "@mui/material/Grid";
import Tooltip from "@mui/material/Tooltip";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Backdrop from "@mui/material/Backdrop";

import OpenCCSwitch from "./OpenCCSwitch";
import CGroupSelect from "./CGroupSelect";
import ConvertButton from "./ConvertButton";

import PACKAGE from "../../package.json";
import { useWasm } from "../WasmContext";

function OptionsControl(
  {
    handleConvert,
  }: {
    handleConvert: (
      target: string,
      mediawiki?: boolean,
      cgroup?: string
    ) => void;
  },
  ref: ForwardedRef<any>
) {
  const { wasm } = useWasm();
  const loading = wasm === null;

  const convertButtonRef = useRef(null as any);
  const isMounting = useRef(true);
  const [cgroups, setCGroups] = useState({} as { [name: string]: string });
  const [activatedCGroups, setActivatedCGroups] = useState(() => {
    return JSON.parse(
      localStorage.getItem(`${PACKAGE.name}-activated-cgroups`) || "[]"
    ) as string[];
  });
  const [wikitextSupport, setWikitextSupport] = useState(() => {
    return JSON.parse(
      localStorage.getItem(`${PACKAGE.name}-wikitext-support`) || "false"
    ) as boolean;
  });
  useEffect(() => {
    async function loadCGroups() {
      const res = await fetch("/cgroups.json");
      const json = await res.json();
      setCGroups(json.data as { [name: string]: string });
    }
    loadCGroups();
  }, []);
  useEffect(() => {
    if (isMounting.current) {
      isMounting.current = false;
      return;
    }
    const s = JSON.stringify(activatedCGroups);
    localStorage.setItem(`${PACKAGE.name}-activated-cgroups`, s);
  }, [activatedCGroups]);
  useEffect(() => {
    if (isMounting.current) {
      // isMounting.current = false;
      return;
    }
    const s = JSON.stringify(wikitextSupport);
    localStorage.setItem(`${PACKAGE.name}-wikitext-support`, s);
    convertButtonRef.current?.click();
  }, [wikitextSupport]);
  return (
    <Box sx={{ position: 'relative' }}>
      <Grid container direction="row" justifyContent="space-around">
        <Grid item>
          <CGroupSelect
            cgroups={Object.keys(cgroups)}
            selected={activatedCGroups}
            onSelect={setActivatedCGroups}
            disabled={loading}
          />
        </Grid>
        <Grid item>
          <Grid
            container
            ref={ref}
            direction="row"
            justifyContent="space-around"
            alignItems="center"
            style={{ alignItems: "center", height: "100%" }}
          >
            <Grid item>
              <Tooltip
                title={
                  <>
                    Enable MediaWiki conversion syntax support
                    <br />/ 啟用 MediaWiki 字詞轉換語法
                  </>
                }
              >
                <FormControlLabel
                  control={
                    <Switch
                      checked={wikitextSupport}
                      onChange={() => setWikitextSupport(!wikitextSupport)}
                      name="mediawiki"
                      color="secondary"
                      disabled={loading}
                    />
                  }
                  label={
                    <Box
                      component="span"
                      display="flex"
                      flexDirection="column"
                      alignItems="center"
                    >
                      <span>Wikitext</span>
                    </Box>
                  }
                />
              </Tooltip>
            </Grid>
            <Grid item>
                <OpenCCSwitch disabled={loading} />
            </Grid>
            <Grid item>
              <ConvertButton
                ref={convertButtonRef}
                onConvert={(target) =>
                  handleConvert(
                    target,
                    wikitextSupport,
                    activatedCGroups.map((name) => cgroups[name]).join("\n")
                  )
                }
                disabled={loading}
              />
            </Grid>
          </Grid>
        </Grid>
      </Grid>
      <Backdrop
        sx={{
          position: 'absolute',
          color: '#fff',
          zIndex: (theme) => theme.zIndex.drawer + 1,
          backgroundColor: 'rgba(0, 0, 0, 0.382)',
        }}
        open={loading}
      >
        <CircularProgress color="secondary" />
      </Backdrop>
    </Box>
  );
}

export default forwardRef(OptionsControl);
