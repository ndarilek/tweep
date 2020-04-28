use crate::Error;
use crate::ErrorList;
use crate::Passage;
use crate::PassageContent;
use crate::Positional;
use crate::Parser;
use crate::Output;
use crate::Warning;
use crate::WarningType;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::default::Default;
use std::collections::HashMap;

/// Represents a full Twee story, but stores the full [`Passage`] object of each
/// field.
///
/// For more information, see the [`Story`] struct.
///
/// [`Passage`]: struct.Passage.html
/// [`Story`]: struct.Story.html
#[derive(Default)]
pub struct StoryPassages {
    /// `StoryTitle` passage
    pub title: Option<Passage>,

    /// `StoryData` passage
    pub data: Option<Passage>,

    /// Map from passage name to `Passage` for any non-special passages
    pub passages: HashMap<String, Passage>,

    /// List of passages tagged with `script`
    pub scripts: Vec<Passage>,

    /// List of passages tagged with `stylesheet`
    pub stylesheets: Vec<Passage>,
}

impl StoryPassages {
    /// Parses an input `String` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_string(input: String) -> Output<Result<Self, ErrorList>> {
        let slice:Vec<&str> = input.split("\n").collect();
        StoryPassages::parse(&slice)
    }

    /// Parses an input `&[&str]` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_slice(input: &[&str]) -> Output<Result<Self, ErrorList>> {
        StoryPassages::parse(input)
    }

    /// Parses a `StoryPassages` from the given [`Path`]. If the given path is
    /// a file, parses that file and returns the `StoryPassages`. If it is a
    /// directory, it looks for any files with `.tw` or `.twee` extensions and
    /// parses them. Returns the parsed output or a list of errors, along with a
    /// list of any [`Warning`]s
    ///
    /// [`Path`]: std::path::Path
    /// [`Warning`]: struct.Warning.html
    pub fn from_path<P: AsRef<Path>>(input: P) -> Output<Result<Self, ErrorList>> {
        let out = StoryPassages::from_path_internal(input);
        let (mut res, mut warnings) = out.take();
        if res.is_ok() {
            let story = res.ok().unwrap();
            let mut story_warnings = story.check();
            warnings.append(&mut story_warnings);
            res = Ok(story);
        }
        Output::new(res).with_warnings(warnings)
    }

    /// Does the heavy lifting for `from_path`. If given a file, reads its
    /// contents into a `String` and uses `from_string` to parse it. If given a
    /// directory, finds the twee files, recurses with each file, then assembles
    /// the outputs into a single output
    fn from_path_internal<P: AsRef<Path>>(input: P) -> Output<Result<Self, ErrorList>> {
        let path:&Path = input.as_ref();
        let path_string:String = path.to_string_lossy().to_owned().to_string();
        if path.is_file() {
            let file_name:String = path.file_name().unwrap().to_string_lossy().to_owned().to_string();
            let file = File::open(path);
            if file.is_err() {
                let err_string = format!("{}", file.err().unwrap());
                return Output::new(Err(Error::new(crate::ErrorType::BadInputPath(path_string, err_string)).into()));
            }
            let mut file = file.ok().unwrap();
            let mut contents = String::new();
            let res = file.read_to_string(&mut contents);
            if res.is_err() {
                let err_string = format!("{}", res.err().unwrap());
                return Output::new(Err(Error::new(crate::ErrorType::BadInputPath(path_string, err_string)).into()));
            }
            StoryPassages::from_string(contents).with_file(file_name)
        } else if path.is_dir() {
            let dir = std::fs::read_dir(path);
            if dir.is_err() {
                let err_string = format!("{}", dir.err().unwrap());
                return Output::new(Err(Error::new(crate::ErrorType::BadInputPath(path_string, err_string)).into()));
            }
            let dir = dir.ok().unwrap();
            let mut story = StoryPassages::default();
            let mut warnings = Vec::new();
            for entry in dir {
                if entry.is_err() {
                    continue;
                }
                let file_path = entry.ok().unwrap().path();
                let extension = file_path.extension();
                if extension.is_none() {
                    continue;
                }
                let extension = extension.unwrap().to_string_lossy();
                if !((extension == "tw" || extension == "twee") && file_path.is_file()) {
                    continue;
                }
                let out = StoryPassages::from_path_internal(file_path);
                let (res, mut sub_warnings) = out.take();
                if res.is_err() {
                    return Output::new(res).with_warnings(warnings);
                }
                let sub_story = res.ok().unwrap();
                let mut merge_warnings = story.merge_from(sub_story);
                warnings.append(&mut sub_warnings);
                warnings.append(&mut merge_warnings);
            }
            Output::new(Ok(story)).with_warnings(warnings)
        } else {
            let err_string = "Path is not a file or directory".to_string();
            Output::new(Err(Error::new(crate::ErrorType::BadInputPath(path_string, err_string)).into()))
        }
    }

    /// Merges the given `StoryPassages` into this one, producing a possible
    /// list of [`Warning`]s in the process.
    ///
    /// # Warnings
    /// Produces a warning if a duplicate `StoryTitle` or `StoryData` is found.
    /// The duplicate is ignored and the existing one is kept.
    pub fn merge_from(&mut self, mut other: Self) -> Vec<Warning> {
        let mut warnings = Vec::new();
        
        match (&self.title, &other.title) {
            (None, Some(_)) => self.title = other.title,
            (Some(_), Some(_)) => {
                let mut warning = Warning::new(WarningType::DuplicateStoryTitle);
                *warning.mut_position() = other.title.unwrap().header.get_position().clone();
                warning.set_referent(self.title.as_ref().unwrap().header.get_position().clone());
                warnings.push(warning)
            },
            _ => (),
        }

        match (&self.data, &other.data) {
            (None, Some(_)) => self.data = other.data,
            (Some(_), Some(_)) => {
                let mut warning = Warning::new(WarningType::DuplicateStoryData);
                *warning.mut_position() = other.data.unwrap().header.get_position().clone();
                warning.set_referent(self.data.as_ref().unwrap().header.get_position().clone());
                warnings.push(warning);
            },
            _ => (),
        }

        self.passages.extend(other.passages);
        self.scripts.append(&mut other.scripts);
        self.stylesheets.append(&mut other.stylesheets);
        
        warnings
    }

    /// Performs a set of post-parse checks and returns a list of any warnings
    ///
    /// # Warnings
    /// * [`MissingStoryTitle`] - No `StoryTitle` passage found
    /// * [`MissingStoryData`] - No `StoryData` passage found
    /// * [`DeadLink`] - Found a link to a non-existent passage
    ///
    /// [`MissingStoryTitle`]: enum.WarningType.html#variant.MissingStoryTitle
    /// [`MissingStoryData`]: enum.WarningType.html#variant.MissingStoryData
    /// [`DeadLink`]: enum.WarningType.html#variant.DeadLink
    pub fn check(&self) -> Vec<Warning> {
        let mut warnings = Vec::new();
        if self.title.is_none() {
            warnings.push(Warning::new(WarningType::MissingStoryTitle));
        }

        if self.data.is_none() {
            warnings.push(Warning::new(WarningType::MissingStoryData));
        }

        for (_, passage) in &self.passages {
            if let PassageContent::Normal(twine) = &passage.content {
                for link in twine.get_links() {
                    if !self.passages.contains_key(&link.target) {
                        warnings.push(Warning {
                            warning_type: WarningType::DeadLink(link.target.clone()),
                            position: link.position.clone(),
                            referent: None,
                        });
                    }
                }
            }
        }

        warnings
    }
}

impl<'a> Parser<'a> for StoryPassages {
    type Output = Output<Result<Self, ErrorList>>;
    type Input = [&'a str];

    fn parse(input: &'a Self::Input) -> Self::Output {
        // The iterator we'll use to walk through the input
        let mut iter = input.iter();
        // The first line must be a header, skip over it so we don't have an
        // empty slice
        iter.next();
        // The starting index of the next passage
        let mut start = 0;

        // Story variables
        let mut title = None;
        let mut data = None;
        let mut passages = HashMap::new();
        let mut scripts = Vec::new();
        let mut stylesheets = Vec::new();

        // Running list of warnings
        let mut warnings = Vec::new();

        // Running list of errors
        let mut errors = Ok(());

        while start < input.len() {
            // Find the start of the next passage using the sigil (::)
            let pos = iter.position(|&x| x.trim_start().starts_with("::"));

            let pos = if pos.is_some() {
                start + pos.unwrap() +1
            } else {
                input.len()
            };
            let passage_input = &input[start..pos];
            println!("Passage input: {:?}", passage_input);

            // Parse the passage
            let (mut res, mut passage_warnings) = Passage::parse(passage_input).with_offset_row(start).take();
            warnings.append(&mut passage_warnings);
            start = pos;

            // If there's an error, update the row before returning
            if res.is_err() {
                errors = ErrorList::merge(&mut errors, &mut res);
                continue;
            }

            let passage = res.ok().unwrap();
            
            // Handle passage types appropriately
            match &passage.content {
                PassageContent::Normal(_) => {
                    passages.insert(passage.header.name.clone(), passage);
                },
                PassageContent::StoryTitle(_) => {
                    if title.is_none() {
                        title = Some(passage);
                    } else {
                        warnings.push(Warning::new(WarningType::DuplicateStoryTitle));
                    }
                },
                PassageContent::StoryData(_, _) => {
                    if data.is_none() {
                        data = Some(passage);
                    } else {
                        warnings.push(Warning::new(WarningType::DuplicateStoryData));
                    }
                },
                PassageContent::Script(_) => scripts.push(passage),
                PassageContent::Stylesheet(_) => stylesheets.push(passage),
            }
        }

        let story = StoryPassages { title, data, passages, scripts, stylesheets };
        Output::new(Ok(story)).with_warnings(warnings)
    }
}

impl Positional for StoryPassages {
    fn set_file(&mut self, file: String) {
        if self.title.is_some() {
            self.title.as_mut().unwrap().set_file(file.clone());
        }

        if self.data.is_some() {
            self.data.as_mut().unwrap().set_file(file.clone());
        }

        for (_, passage) in &mut self.passages {
            passage.set_file(file.clone());
        }

        for script in &mut self.scripts {
            script.set_file(file.clone());
        }

        for style in &mut self.stylesheets {
            style.set_file(file.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::Warning;
    use crate::WarningType;

    #[test]
    fn warning_offsets() {
        let input = r#":: A passage
This
That
The Other


:: A\[nother passage
Foo
Bar
Baz


:: StoryTitle
Test Story


"#.to_string();
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        assert_eq!(warnings[0], Warning::new(WarningType::EscapedOpenSquare).with_row(6).with_column(4));
    }

    #[test]
    fn file_input() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#":: A passage
This
That
The Other


:: A\[nother passage
Foo
Bar
Baz


:: StoryTitle
Test Story


"#.to_string();
        use std::io::{Write};
        let dir = tempdir()?;
        let file_path = dir.path().join("test.twee");
        let mut file = File::create(file_path.clone())?;
        writeln!(file, "{}", input)?;

        let out = StoryPassages::from_path(file_path);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
            assert_eq!(warnings[0], Warning::new(WarningType::EscapedOpenSquare).with_row(6).with_column(4).with_file("test.twee".to_string()));
            assert_eq!(warnings[1], Warning::new(WarningType::MissingStoryData));
        } else {
            panic!("Expected StoryTitle");
        }

        Ok(())
    }

    #[test]
    fn a_test() {
        let input = r#":: A passage
This
That
The Other


:: Another passage
Foo
Bar
Baz


:: StoryTitle
Test Story


"#.to_string();
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
        } else {
            panic!("Expected StoryTitle");
        }
    }

    #[test]
    fn dead_link() {
        let input = r#":: A passage
This passage links to [[Another passage]]

:: Another passage
This has dead link to [[Dead link]]

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "abc"
}
"#.to_string();
        let out = StoryPassages::from_string(input);
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert_eq!(warnings, vec![Warning::new(
            WarningType::DeadLink("Dead link".to_string()))
                                  .with_row(4)
                                  .with_column(24)
        ]);
    }
}
