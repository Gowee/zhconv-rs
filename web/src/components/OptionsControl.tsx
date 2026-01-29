import { forwardRef, ForwardedRef, useState, useEffect, useRef } from "react";
import FormControlLabel from "@mui/material/FormControlLabel";
import Switch from "@mui/material/Switch";
import Grid from "@mui/material/Grid";
// import ChangeCircleOutlinedIcon from '@mui/icons-material/ChangeCircleOutlined';
import Tooltip /*, { TooltipProps }*/ from "@mui/material/Tooltip";
import Box from "@mui/material/Box";

import CGroupSelect from "./CGroupSelect";
import ConvertButton from "./ConvertButton";

import PACKAGE from "../../package.json";

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
  const convertButtonRef = useRef(null as any);
  const isMounting = useRef(true);
  const [cgroups, setCGroups] = useState({} as { [name: string]: string });
  const [activatedCGroups, setActivatedCGroups] = useState(() => {
    return JSON.parse(
      localStorage.getItem(`${PACKAGE.name}-activated-cgroups`) || "[]"
    ) as string[];
  });
  const [parsingInline, setParsingInline] = useState(() => {
    return JSON.parse(
      localStorage.getItem(`${PACKAGE.name}-parsing-inline`) || "false"
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
    const s = JSON.stringify(parsingInline);
    localStorage.setItem(`${PACKAGE.name}-parsing-inline`, s);
    convertButtonRef.current?.click();
  }, [parsingInline]);
  return (
    <Grid container direction="row" justifyContent="space-around">
      <Grid item>
        <CGroupSelect
          cgroups={Object.keys(cgroups)}
          selected={activatedCGroups}
          onSelect={setActivatedCGroups}
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
                  Parse and apply inline rules in MediaWiki LanguageConverter
                  syntax
                  <br />/ 解析並應用文本中的 MediaWiki 語言轉換規則
                </>
              }
            >
              <FormControlLabel
                control={
                  <Switch
                    checked={parsingInline}
                    onChange={() => setParsingInline(!parsingInline)}
                    name="mediawiki"
                    color="secondary"
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
            <ConvertButton
              ref={convertButtonRef}
              onConvert={(target) =>
                handleConvert(
                  target,
                  parsingInline,
                  activatedCGroups.map((name) => cgroups[name]).join("\n")
                )
              }
            />
          </Grid>
        </Grid>
      </Grid>
    </Grid>
  );
}

export default forwardRef(OptionsControl);
