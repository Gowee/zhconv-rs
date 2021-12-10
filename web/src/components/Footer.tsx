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
          </Typography>
        </Grid>
      </Grid>
    </footer>
  );
}
