import { forwardRef, ForwardedRef, useState, useEffect } from "react";

import { withStyles, Theme, makeStyles } from "@material-ui/core/styles";
import FormGroup from "@material-ui/core/FormGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Switch from "@material-ui/core/Switch";
import Grid from "@material-ui/core/Grid";
import ButtonGroup from "@material-ui/core/ButtonGroup";
import Button from "@material-ui/core/Button";
import InputLabel from "@material-ui/core/InputLabel";
import MenuItem from "@material-ui/core/MenuItem";
import FormHelperText from "@material-ui/core/FormHelperText";
import FormControl from "@material-ui/core/FormControl";
import Select from "@material-ui/core/Select";
// import ChangeCircleOutlinedIcon from '@material-ui/icons/ChangeCircleOutlined';
import Tooltip, { TooltipProps } from "@material-ui/core/Tooltip";

import CGroupSelect from "./CGroupSelect";
import ConvertButton from "./ConvertButton";

const LightTooltip = withStyles((theme: Theme) => ({
  tooltip: {
    backgroundColor: theme.palette.common.white,
    color: "rgba(0, 0, 0, 0.87)",
    boxShadow: theme.shadows[1],
    fontSize: 11,
  },
}))(Tooltip);

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
  const [activatedCGroups, setActivatedCGroups] = useState([] as string[]);
  const [parsingInline, setParsingInline] = useState(false);
  useEffect(() => {
    async function loadCGroups() {
      const { default: d } = await import("../../public/cgroups.json");
      setCGroups(d);
    }
    loadCGroups();
  }, []);
  useEffect(() => console.log("bem", activatedCGroups), [activatedCGroups]);
  // console.log(cgroups);
  console.log(activatedCGroups);
  return (
    <Grid container direction="column">
      <Grid item>
        <CGroupSelect
          cgroups={Object.keys(cgroups)}
          selected={activatedCGroups}
          onSelect={setActivatedCGroups}
        />
      </Grid>
      <Grid item>
        {/* {" "} */}
        <Grid
          container
          ref={ref}
          direction="row"
          justifyContent="space-around"
          alignItems="center"
          style={{ alignItems: "center", height: "100%" }}
        >
          {/* <Grid item>
            <FormControl variant="outlined" margin="dense">
              <InputLabel id="target-label">Target / 目標</InputLabel>
              <Select
                labelId="target-label"
                id="target-select"
                value={"zh-TW"}
              // onChange={handleChange}
              >
                <MenuItem value="zh">zh / 原文</MenuItem>
                <MenuItem value="zh-hant">zh-Hant 繁體</MenuItem>
                <MenuItem value="zh-hans">zh-Hans 简体</MenuItem>
                <MenuItem value="zh-TW">zh-TW 臺灣正體</MenuItem>
                <MenuItem value="zh-HK">zh-HK 香港繁體</MenuItem>
                <MenuItem value="zh-MO">zh-MO 澳門繁體</MenuItem>
                <MenuItem value="zh-CN">zh-CN 大陆简体</MenuItem>
                <MenuItem value="zh-SG">zh-SG 大陆简体</MenuItem>
                <MenuItem value="zh-MY">zh-MY 大陆简体</MenuItem>
              </Select>
            </FormControl>
          </Grid> */}
          <Grid item>
            <Tooltip title="Parse MediaWiki conversion rules in the text">
              <FormControlLabel
                control={
                  <Switch
                    checked={parsingInline}
                    onChange={() => setParsingInline(!parsingInline)}
                    name="mediawiki"
                  />
                }
                label="Inline Rules"
              />
            </Tooltip>
            {/* <ButtonGroup color="primary" aria-label="control button group">
              <Button
                color="primary"
                onClick={() =>
                  handleConvert(
                    "zh-tw",
                    true,
                    activatedCGroups.map((k) => cgroups[k]).join("\n")
                  )
                }
              >
                Convert 轉換
              </Button>
              <Button color="primary" onClick={() => handleConvert("zh-cn")}>
                Reverse
              </Button>
            </ButtonGroup> */}
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
          {/* <Grid item> */}
          {/* <FormGroup row>
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv6"}
                onChange={toggleIpv4}
                name="ipv4"
              />
            }
            label="IPv4"
          />
          <FormControlLabel
            control={
              <Switch
                checked={ipKind !== "ipv4"}
                onChange={toggleIpv6}
                name="ipv6"
              />
            }
            label="IPv6"
          />
        </FormGroup> */}
          {/* </Grid>
          <Grid item> */}
          {/* <Tooltip title="If activated, all reserved IPs for special purposes (RFC 5735 and RFC 6890) are filtered out from the output. This might be useful when reversing.">
          <FormControlLabel
            control={
              <Switch
                checked={bogonFilter === "reserved"}
                onChange={toggleReservedFilter}
                name="excludeReserved"
              />
            }
            label="Exclude reserved IPs"
          />
        </Tooltip> */}
          {/* </Grid> */}
        </Grid>
      </Grid>
    </Grid>
  );
}

export default forwardRef(OptionsControl);
