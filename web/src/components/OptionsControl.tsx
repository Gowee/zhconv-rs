import { forwardRef, ForwardedRef, useState, useEffect } from "react";

// import { withStyles, Theme, makeStyles } from "@material-ui/core/styles";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Switch from "@material-ui/core/Switch";
import Grid from "@material-ui/core/Grid";
// import ChangeCircleOutlinedIcon from '@material-ui/icons/ChangeCircleOutlined';
import Tooltip /*, { TooltipProps }*/ from "@material-ui/core/Tooltip";
import Box from "@material-ui/core/Box";

import CGroupSelect from "./CGroupSelect";
import ConvertButton from "./ConvertButton";

import PACKAGE from "../../package.json";

// const LightTooltip = withStyles((theme: Theme) => ({
//   tooltip: {
//     backgroundColor: theme.palette.common.white,
//     color: "rgba(0, 0, 0, 0.87)",
//     boxShadow: theme.shadows[1],
//     fontSize: 11,
//   },
// }))(Tooltip);

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
      const { data } = await import("../../public/cgroups.json");
      setCGroups(data);
    }
    loadCGroups();
  }, []);
  useEffect(() => {
    const s = JSON.stringify(activatedCGroups);
    localStorage.setItem(`${PACKAGE.name}-activated-cgroups`, s);
  }, [activatedCGroups]);
  useEffect(() => {
    const s = JSON.stringify(parsingInline);
    localStorage.setItem(`${PACKAGE.name}-parsing-inline`, s);
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
                  Parse MediaWiki conversion rules in the text
                  <br />/ 解析文本中的 MediaWiki 轉換規則
                </>
              }
            >
              <FormControlLabel
                control={
                  <Switch
                    checked={parsingInline}
                    onChange={() => setParsingInline(!parsingInline)}
                    name="mediawiki"
                  />
                }
                label={
                  <Box
                    component="span"
                    display="flex"
                    flexDirection="column"
                    alignItems="center"
                  >
                    <span>Inline Rules</span>
                    <span>文內規則</span>
                  </Box>
                }
              />
            </Tooltip>
          </Grid>
          <Grid item>
            <ConvertButton
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
