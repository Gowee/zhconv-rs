import { useEffect, useState } from "react";
import { makeStyles } from "@material-ui/core/styles";
import Grid from "@material-ui/core/Grid";
import Typography from "@material-ui/core/Typography";
import Link from "@material-ui/core/Link";
import GitHubIcon from "@material-ui/icons/GitHub";

const useStyles = makeStyles((theme) => ({
  root: {
    marginTop: theme.spacing(3),
  },
  icon: {
    fontSize: "1rem",
  },
}));

export default function Footer() {
  const classes = useStyles();
  const [buildInfo, setBuildInfo] = useState(
    {} as {
      buildDate?: Date;
      commit?: string;
      mediawikiCommit?: string;
      openccCommit?: string;
      cgroupDate?: Date;
    }
  );
  useEffect(() => {
    async function loadBuildInfo() {
      const {
        get_build_timestamp,
        get_commit,
        get_mediawiki_commit,
        get_opencc_commit,
      } = await import("../../../pkg/zhconv.js");
      const { timestamp: cgroupTimestamp } = await import(
        "../../public/cgroups.json"
      );
      setBuildInfo({
        buildDate: new Date(get_build_timestamp() ?? 0),
        commit: get_commit(),
        mediawikiCommit: get_mediawiki_commit(),
        openccCommit: get_opencc_commit(),
        cgroupDate: new Date(cgroupTimestamp * 1000),
      });
    }
    loadBuildInfo();
  }, []);

  return (
    <footer className={classes.root}>
      <Grid container justifyContent="space-between">
        <Grid item>
          <Typography variant="body2" color="textSecondary">
            <Link color="inherit" href="https://github.com/Gowee/zhconv-rs">
              <GitHubIcon className={classes.icon} />
              {" Source code"}
            </Link>
          </Typography>
        </Grid>
        <Grid item>
          <Typography variant="body2" color="textSecondary">
            {"Build: "}
            <Link
              color="inherit"
              href={`https://github.com/Gowee/zhconv-rs/commit/${
                buildInfo.commit ?? ""
              }`}
              underline="always"
              title={buildInfo?.buildDate?.toLocaleString() ?? undefined}
            >
              <code>{buildInfo.commit?.substring(0, 8) ?? "10A0D149."}</code>
            </Link>
            {" | "}
            {"MediaWiki: "}
            <Link
              color="inherit"
              href={`https://github.com/wikimedia/mediawiki/blob/${
                buildInfo.mediawikiCommit ?? "master"
              }/includes/languages/data/ZhConversion.php#L14`}
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
              href={`https://github.com/BYVoid/OpenCC/blob/${
                buildInfo.openccCommit ?? "master"
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
              {buildInfo.cgroupDate?.toLocaleString() ?? "Date unknown"}
            </Link>
          </Typography>
          {/* <Typography variant="body2" color="textSecondary">
            {"Based on conversion tables from "}
            <Link
              color="inherit"
              href="https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/languages/data/ZhConversion.php"
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
    </footer>
  );
}
