# Bevy Flappy Bird

A reimplementation of Flappy Bird using the Bevy game framework.

## USAGE

Run `./source_assets.sh` to download .APK + .ZIP sources and extract required assets to `./assets` directory.

```
Usage: ./source_assets.sh [options]

Options:
  --force-download   Re-download source archives even if valid cached copies exist
  --clean            Delete ./temp after successful extraction
  --overwrite        Overwrite all extracted files without prompting
  --help             Show this help message
```

Downloaded source files include the following:
| | |
| :-- | :-- |
| NAME | com.dotgears.flappybird-1.3-4-minAPI8.apk |
| SIZE | 894 KB |
| MD5 | BF978C69C8E594E6FE301B677E69ACBC |
| SHA1 | 9F472383AA7335AF4E963635D496D606CEA56622 |
| SHA256 | A3E6958CE2100966F4E207778E4CDBE72788214148C7F4BFD042BA365498DEB3 |
| SHA512 | 6F6819D07F8342599C756797E4EB71200CAC02609C3DD57E5E03B69B9DB0420B81BFAA35AAACFD49056BDEFCD40EA84DDCB5350ADA1482A84D9327022920CEAA |

| | |
| :-- | :-- |
| NAME | paulkr_Flappy-Bird.zip |
| SIZE | 406 KB |
| MD5 | 19E22337C7DAFA9DD2B6522119ACDE1C |
| SHA1 | 1A7F8D4B1990DB46E98B5707B7F7B22D86FF8EC4 |
| SHA256 | EF9C0B3C1885A92DDA160732FF166F967149960DF693743623DABE4C75D704A1 |
| SHA512 | 68917E20874BBA1AADF9F2404F98FF11D54216812F4B3E80230116DD985F2CEF55C91D2867087C9DC7E23BDDA84F12553C3C010123934599E79B407273811AD6 |

Once `./assets` is populated with the required files,<br>
run `cargo run` to test.
