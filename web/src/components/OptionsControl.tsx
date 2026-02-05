import { forwardRef, ForwardedRef, useRef, useImperativeHandle } from "react";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";
import Grid from "@mui/material/Grid";
import Tooltip from "@mui/material/Tooltip";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Backdrop from "@mui/material/Backdrop";
import { useTheme } from "@mui/material/styles";
import useMediaQuery from "@mui/material/useMediaQuery";
import Divider from "@mui/material/Divider";

import RulesetSelector from "./RulesetSelector";
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
    cgroups: string[] | null;
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
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
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
      <Grid container direction={"row"} alignItems="center" flexWrap={{ xs: "wrap", md: "nowrap" }} justifyContent={{ xs: "center", md: "flex-start" }}>
        <Grid container flexGrow={1} flexShrink={1} direction="row" justifyContent="space-around" alignSelf="stretch" alignItems="center" sx={{ minWidth: 0, order: { xs: 2, md: 1 } }}>
          <Grid>
            <CGroupSelect
              cgroups={cgroups}
              selected={activatedCGroups}
              onSelect={onSelectCGroups}
              disabled={loading}
            />
          </Grid>
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
        <Divider
          orientation={isMobile ? "horizontal" : "vertical"}
          variant={isMobile ? "fullWidth" : "middle"}
          flexItem
          sx={{
            mx: { xs: 0, md: 1 },
            my: { xs: 0.5, md: 0 },
            width: { xs: '100%', md: 'auto' },
            order: { xs: 1, md: 2 }
          }}
        />
        <Grid sx={{
          flexBasis: { xs: '100%', md: 'auto' },
          flexShrink: 0,
          display: 'flex',
          justifyContent: 'center',
          mt: { xs: 0, md: 0 },
          mb: { xs: 0.5, md: 0 },
          order: { xs: 0, md: 3 }
        }}>
          <RulesetSelector disabled={loading} />
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
    </Box >
  );
}

export default forwardRef(OptionsControl);
