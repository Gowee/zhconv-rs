import { useEffect, useState } from "react";
import Box from "@mui/material/Box";
import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
import Link from "@mui/material/Link";
import GitHubIcon from "@mui/icons-material/GitHub";
import { useApp } from "../AppContext";

export default function Footer() {
  const { wasm, cgroups } = useApp();
  const [buildInfo, setBuildInfo] = useState(
    {} as {
      buildDate?: Date;
      commit?: string;
      mediawikiCommit?: string;
      openccCommit?: string;
      cgroupDate?: Date;
    },
  );
  useEffect(() => {
    async function loadBuildInfo() {
      if (!wasm) return;
      setBuildInfo({
        buildDate: new Date(wasm.get_build_timestamp() ?? 0),
        commit: wasm.get_commit(),
        mediawikiCommit: wasm.get_mediawiki_commit(),
        openccCommit: wasm.get_opencc_commit(),
        cgroupDate: cgroups.timestamp ? new Date(cgroups.timestamp * 1000) : undefined,
      });
    }
    loadBuildInfo();
  }, [wasm, cgroups]);

  return (
    <Box component="footer" sx={{ mt: 3 }}>
      <Grid container justifyContent="space-between">
        <Grid item>
          <Typography variant="body2" color="textSecondary">
            <Link color="inherit" href="https://github.com/Gowee/zhconv-rs">
              <GitHubIcon sx={{ fontSize: "1rem" }} />
              {" Source code"}
            </Link>
          </Typography>
        </Grid>
        <Grid item>
          <Typography variant="body2" color="textSecondary">
            {"Build: "}
            <Link
              color="inherit"
              href={`https://github.com/Gowee/zhconv-rs/commit/${buildInfo.commit ?? ""
                }`}
              underline="always"
              title={buildInfo?.buildDate?.toISOString() ?? undefined}
            >
              <code>{buildInfo.commit?.substring(0, 8) ?? "10A0D149."}</code>
            </Link>
            {" | "}
            {"MediaWiki: "}
            <Link
              color="inherit"
              href={`https://github.com/wikimedia/mediawiki/blob/${buildInfo.mediawikiCommit ?? "master"
                }/includes/Languages/Data/ZhConversion.php#L14`}
              underline="always"
            >
              <code>
                {buildInfo.mediawikiCommit?.substring(0, 8) ?? "????????"}
              </code>
            </Link>
            {" | "}
            {"OpenCC: "}
            <Link
              color="inherit"
              href={`https://github.com/BYVoid/OpenCC/blob/${buildInfo.openccCommit ?? "master"
                }/data/dictionary`}
              underline="always"
            >
              <code>
                {buildInfo.openccCommit?.substring(0, 8) ??
                  (buildInfo.mediawikiCommit ? "__N.A.__" : "????????")}
              </code>
            </Link>
            {" | "}
            {"CGroups: "}
            <Link
              color="inherit"
              href={`https://zh.wikipedia.org/wiki/Template:CGroup/list`}
              underline="always"
            >
              {buildInfo.cgroupDate?.toISOString() ?? "DATE_UNKNOWN"}
            </Link>
          </Typography>
          {/* <Typography variant="body2" color="textSecondary">
            {"Based on conversion tables from "}
            <Link
              color="inherit"
              href="https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/languages/Data/ZhConversion.php"
              underline="always"
            >
              {"MediaWiki"}
            </Link>
            {" and "}
            <Link
              color="inherit"
              href="https://zh.wikipedia.org/wiki/Module:CGroup"
              underline="always"
            >
              {"Chinese Wikipedia"}
            </Link>
            {"."}
          </Typography> */}
        </Grid>
      </Grid>
    </Box>
  );
}
