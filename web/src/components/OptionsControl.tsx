import {
  forwardRef,
  ForwardedRef,
  useRef,
  useImperativeHandle,
} from "react";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";
import Grid from "@mui/material/Grid";
import Tooltip from "@mui/material/Tooltip";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Backdrop from "@mui/material/Backdrop";

import OpenCCSwitch from "./OpenCCSwitch";
import CGroupSelect from "./CGroupSelect";
import ConvertButton, { Variant } from "./ConvertButton";

import { useApp } from "../AppContext";

export interface OptionsControlHandle {
  controlElement: HTMLDivElement | null;
  clickConvert: () => void;
}

function OptionsControl(
  {
    cgroups,
    activatedCGroups,
    onSelectCGroups,
    wikitextSupport,
    onToggleWikitextSupport,
    onConvert,
    targetVariant,
    setTargetVariant,
  }: {
    cgroups: string[];
    activatedCGroups: string[];
    onSelectCGroups: (groups: string[]) => void;
    wikitextSupport: boolean;
    onToggleWikitextSupport: () => void;
    onConvert: () => void;
    targetVariant: Variant;
    setTargetVariant: (target: Variant) => void;
  },
  ref: ForwardedRef<OptionsControlHandle>,
) {
  const { wasm } = useApp();
  const loading = wasm === null;
  const controlDivRef = useRef<HTMLDivElement>(null);
  const convertButtonRef = useRef<HTMLButtonElement>(null);

  useImperativeHandle(ref, () => ({
    controlElement: controlDivRef.current,
    clickConvert: () => {
      convertButtonRef.current?.click();
    },
  }));

  return (
    <Box className="options-control" sx={{ position: "relative" }}>
      <Grid container direction="row" justifyContent="space-around">
        <Grid>
          <CGroupSelect
            cgroups={cgroups}
            selected={activatedCGroups}
            onSelect={onSelectCGroups}
            disabled={loading}
          />
        </Grid>
        <Grid>
          <Grid
            container
            ref={controlDivRef}
            direction="row"
            justifyContent="space-around"
            alignItems="center"
            style={{ alignItems: "center", height: "100%" }}
          >
            <Grid>
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
                      onChange={onToggleWikitextSupport}
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
            <Grid>
              <OpenCCSwitch disabled={loading} />
            </Grid>
            <Grid>
              <ConvertButton
                ref={convertButtonRef}
                onConvert={onConvert}
                targetVariant={targetVariant}
                setTargetVariant={setTargetVariant}
                disabled={loading}
              />
            </Grid>
          </Grid>
        </Grid>
      </Grid>
      <Backdrop
        sx={{
          position: "absolute",
          color: "#fff",
          zIndex: (theme) => theme.zIndex.drawer + 1,
          backgroundColor: "rgba(0, 0, 0, 0.382)",
        }}
        open={loading}
      >
        <CircularProgress color="secondary" />
      </Backdrop>
    </Box>
  );
}

export default forwardRef(OptionsControl);