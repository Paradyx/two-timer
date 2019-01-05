/*!

This crate provides a `parse` function to convert English time expressions into a pair
of timestamps representing a time range. It converts "today" into the first and last
moments of today, "May 6, 1968" into the first and last moments of that day, "last year"
into the first and last moments of that year, and so on. It does this even for expressions
generally interpreted as referring to a point in time, such as "3 PM". In these cases
the width of the time span varies according to the specificity of the expression. "3 PM" has
a granularity of an hour, "3:00 PM", of a minute, "3:00:00 PM", of a second. For pointwise
expression the first moment is the point explicitly named. The `parse` expression actually
returns a 3-tuple consisting of the two timestamps and whether the expression is literally
a range -- two time expressions separated by a preposition such as "to", "through", "up to",
or "until".

# Example

```rust
extern crate two_timer;
use two_timer::{parse, Config};
extern crate chrono;
use chrono::naive::NaiveDate;

pub fn main() {
    let phrases = [
        "now",
        "this year",
        "last Friday",
        "from now to the end of time",
        "Ragnarok",
        "at 3:00 pm today",
        "5/6/69",
        "Tuesday, May 6, 1969 at 3:52 AM",
        "March 15, 44 BC",
    ];
    // find the maximum phrase length for pretty formatting
    let max = phrases
        .iter()
        .max_by(|a, b| a.len().cmp(&b.len()))
        .unwrap()
        .len();
    for phrase in phrases.iter() {
        match parse(phrase, None) {
            Ok((d1, d2, _)) => println!("{:width$} => {} --- {}", phrase, d1, d2, width = max),
            Err(e) => println!("{:?}", e),
        }
    }
    let now = NaiveDate::from_ymd_opt(1066, 10, 14).unwrap().and_hms(12, 30, 15);
    println!("\nlet \"now\" be some moment during the Battle of Hastings, specifically {}\n", now);
    let conf = Config::new().now(now);
    for phrase in phrases.iter() {
        match parse(phrase, Some(conf.clone())) {
            Ok((d1, d2, _)) => println!("{:width$} => {} --- {}", phrase, d1, d2, width = max),
            Err(e) => println!("{:?}", e),
        }
    }
}
```
produces
```text
now                             => 2018-12-29 21:56:00 UTC --- 2018-12-29 21:57:00 UTC
this year                       => 2018-01-01 00:00:00 UTC --- 2019-01-01 00:00:00 UTC
last Friday                     => 2018-12-21 00:00:00 UTC --- 2018-12-22 00:00:00 UTC
from now to the end of time     => 2018-12-29 21:56:00 UTC --- +262143-12-31 23:59:59.999 UTC
Ragnarok                        => +262143-12-31 23:59:59.999 UTC --- +262143-12-31 23:59:59.999 UTC
at 3:00 pm today                => 2018-12-29 15:00:00 UTC --- 2018-12-29 15:01:00 UTC
5/6/69                          => 1969-05-06 00:00:00 UTC --- 1969-05-07 00:00:00 UTC
Tuesday, May 6, 1969 at 3:52 AM => 1969-05-06 03:52:00 UTC --- 1969-05-06 03:53:00 UTC
March 15, 44 BC                 => -0043-03-15 00:00:00 UTC --- -0043-03-16 00:00:00 UTC

let "now" be some moment during the Battle of Hastings, specifically 1066-10-14 12:30:15 UTC

now                             => 1066-10-14 12:30:00 UTC --- 1066-10-14 12:31:00 UTC
this year                       => 1066-01-01 00:00:00 UTC --- 1067-01-01 00:00:00 UTC
last Friday                     => 1066-10-05 00:00:00 UTC --- 1066-10-06 00:00:00 UTC
from now to the end of time     => 1066-10-14 12:30:00 UTC --- +262143-12-31 23:59:59.999 UTC
Ragnarok                        => +262143-12-31 23:59:59.999 UTC --- +262143-12-31 23:59:59.999 UTC
at 3:00 pm today                => 1066-10-14 15:00:00 UTC --- 1066-10-14 15:01:00 UTC
5/6/69                          => 0969-05-06 00:00:00 UTC --- 0969-05-07 00:00:00 UTC
Tuesday, May 6, 1969 at 3:52 AM => 1969-05-06 03:52:00 UTC --- 1969-05-06 03:53:00 UTC
March 15, 44 BC                 => -0043-03-15 00:00:00 UTC --- -0043-03-16 00:00:00 UTC
```

# Relative Times

It is common in English to use time expressions which must be interpreted relative to some
context. The context may be verb tense, other events in the discourse, or other semantic or
pragmatic clues. The `two_timer` `parse` function doesn't attempt to infer context perfectly, but
it does make some attempt to get the context right. So, for instance "last Monday through Friday", said
on Saturday, will end on a different day from "next Monday through Friday". The general rules
are

1. a fully-specified expression in a pair will provide the context for the other expression
2. a relative expression will be interpreted as appropriate given its order -- the second expression
describes a time after the first
3. if neither expression is fully-specified, the first will be interpreted relative to "now" and the
second relative ot the first

The rules of interpretation for relative time expressions in ranges will likely be refined further
in the future.

# Clock Time

The parse function interprets expressions such as "3:00" as referring to time on a 24 hour clock, so
"3:00" will be interpreted as "3:00 AM". This is true even in ranges such as "3:00 PM to 4", where the
more natural interpretation might be "3:00 PM to 4:00 PM".

# Years Near 0

Since it is common to abbreviate years to the last two digits of the century, two-digit
years will be interpreted as abbreviated unless followed by a suffix such as "B.C.E." or "AD".
They will be interpreted as the the nearest appropriate *previous* year to the current moment,
so in 2010 "'11" will be interpreted as 1911, not 2011.

# The Second Time in Ranges

For single expressions, like "this year", "today", "3:00", or "next month", the second of the
two timestamps is straightforward -- it is the end of the relevant temporal unit. "1971" will
be interpreted as the first moment of the first day of 1971 through, but excluding, the first
moment of the first day of 1972, so the second timestamp will be this first excluded moment.

When the parsed expression describes a range, we're really dealing with two potentially overlapping
pairs of timestamps and the choice of the terminal timestamp gets trickier. The general rule
will be that if the second interval is shorter than a day, the first timestamp is the first excluded moment,
so "today to 3:00 PM" means the first moment of the day up to, but excluding, 3:00 PM. If the second
unit is as big as or larger than a day, which timestamp is used varies according to the preposition.
"This week up to Friday" excludes all of Friday. "This week through Friday" includes all of Friday.
Prepositions are assumed to fall into either the "to" class or the "through" class. You may also use
a series of dashes as a synonym for "through", so "this week - fri" is equivalent to "this week through Friday".
For the most current list of prepositions in each class, consult the grammar used for parsing, but
as of the moment, these are the rules:

```text
        up_to => [["to", "until", "up to", "till"]]
        through => [["up through", "through", "thru"]] | r("-+")
```

# Pay Periods

I'm writing this library in anticipation of, for the sake of amusement, rewriting [JobLog](https://metacpan.org/pod/App::JobLog)
in Rust. This means I need the time expressions parsed to include pay periods. Pay periods, though,
are defined relative to some reference date -- a particular Sunday, say -- and have a variable period.
`two_timer`, and JobLog, assume pay periods are of a fixed length and tile the timeline without overlap, so a
pay period of a calendrical month is problematic.

If you need to interpret "last pay period", say, you will need to specify when this pay period began, or
when some pay period began or will begin, and a pay period length in days. The `parse` function has a second
optional argument, a `Config` object, whose chief function outside of testing is to provide this information. So,
for example, you could do this:

```rust
# extern crate two_timer;
# use two_timer::{parse, Config};
# extern crate chrono;
# use chrono::{Date, TimeZone, Utc};
let (reference_time, _, _) = parse("5/6/69", None).unwrap();
let config = Config::new().pay_period_start(Some(reference_time.date()));
let (t1, t2, _) = parse("next pay period", Some(config)).unwrap();
```

# Ambiguous Year Formats

`two_timer` will try various year-month-day permutations until one of them parses given that days are in the range 1-31 and
months, 1-12. This is the order in which it tries these permutations:

1. year/month/day
2. year/day/month
3. month/day/year
4. day/month/year

The potential unit separators are `/`, `.`, and `-`. Whitespace is optional.

# Timezones

At the moment `two_timer` only knows about UTC time. Sorry about that.

*/

#![recursion_limit = "1024"]
#[macro_use]
extern crate pidgin;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
use chrono::naive::{NaiveDate, NaiveDateTime};
use chrono::{Date, DateTime, Datelike, Duration, Local, TimeZone, Timelike, Weekday};
use pidgin::{Match, Matcher};
use regex::Regex;

lazy_static! {
    static ref MATCHER: Matcher = grammar!{
        (?ibBw)

        TOP -> r(r"\A") <something> r(r"\z")

        something => <universal> | <particular>
        universal => [["always", "ever", "all time", "forever", "from beginning to end", "from the beginning to the end"]]
        particular => <one_time> | <two_times>
        one_time => <moment_or_period>
        two_times -> ("from")? <moment_or_period> <to> <moment_or_period>
        to => <up_to> | <through>
        up_to => [["to", "until", "up to", "till"]]
        through => [["up through", "through", "thru"]] | r("-+")
        moment_or_period => <moment> | <period>
        period => <named_period> | <specific_period>
        specific_period => <modified_period> | <month_and_year>
        month_and_year -> <a_month> <year>
        named_period => <a_day> | <a_month>
        modified_period -> <modifier> <modifiable_period>
        modifier => [["this", "last", "next"]]
        modifiable_period => [["week", "month", "year", "pay period", "pp", "weekend"]] | <a_month> | <a_day>
        moment -> <at_time_on>? <some_day> <at_time>? | <specific_time> | <time>
        specific_time => <first_time> | <last_time>
        some_day => <specific_day> | <relative_day>
        specific_day => <adverb> | <date_with_year>
        relative_day => <a_day>
        adverb => [["now", "today", "tomorrow", "yesterday"]]
        date_with_year => <n_date> | <a_date>
        at_time -> ("at") <time>
        at_time_on -> ("at")? <time> ("on")?
        time -> <hour_12> <am_pm>? | <hour_24>
        hour_24 => <h24>
        hour_24 => <h24> (":") <minute>
        hour_24 => <h24> (":") <minute> (":") <second>
        hour_12 => <h12>
        hour_12 => <h12> (":") <minute>
        hour_12 => <h12> (":") <minute> (":") <second>
        minute => [ (0..60).into_iter().map(|i| format!("{:02}", i)).collect::<Vec<_>>() ]
        second => [ (0..60).into_iter().map(|i| format!("{:02}", i)).collect::<Vec<_>>() ]
        am_pm => (?-i) [["am", "AM", "pm", "PM", "a.m.", "A.M.", "p.m.", "P.M."]]
        h12 => [(1..=12).into_iter().collect::<Vec<_>>()]
        h24 => [(1..=24).into_iter().collect::<Vec<_>>()]
        n_date -> <year>    r("[./-]") <n_month> r("[./-]") <n_day>
        n_date -> <year>    r("[./-]") <n_day>   r("[./-]") <n_month>
        n_date -> <n_month> r("[./-]") <n_day>   r("[./-]") <year>
        n_date -> <n_day>   r("[./-]") <n_month> r("[./-]") <year>
        a_date -> <a_month> <n_day> (",") <year>
        a_date -> <n_day> <a_month> <year>
        a_date -> <a_day> (",") <a_month> <n_day> (",") <year>
        year => <short_year> | ("-")? <n_year>
        year -> <suffix_year> <year_suffix>
        short_year => [
                (0..=99)
                    .into_iter()
                    .flat_map(|i| vec![format!("'{:02}", i), format!("{:02}", i)])
                    .collect::<Vec<_>>()
            ]
        n_year => r(r"\b(?:[1-9][0-9]{0,4}|0)\b")
        suffix_year => r(r"\b[1-9][0-9]{0,4}\b")
        year_suffix => <ce> | <bce>
        ce => (?-i) [["ce", "c.e.", "ad", "a.d.", "CE", "C.E.", "AD", "A.D."]]
        bce => (?-i) [["bce", "b.c.e.", "bc", "b.c.", "BCE", "B.C.E.", "BC", "B.C."]]
        n_day => [
                (1..=31)
                    .into_iter()
                    .flat_map(|i| vec![i.to_string(), format!("{:02}", i)])
                    .collect::<Vec<_>>()
            ]
        n_month => [
                (1..12)
                    .into_iter()
                    .flat_map(|i| vec![format!("{:02}", i), format!("{}", i)])
                    .collect::<Vec<_>>()
            ]
        a_day => [
                "Sunday Monday Tuesday Wednesday Thursday Friday Saturday Tues Weds Thurs Tues. Weds. Thurs."
                    .split(" ")
                    .into_iter()
                    .flat_map(|w| vec![
                        w.to_string(),
                        w[0..2].to_string(),
                        w[0..3].to_string(),
                        format!("{}.", w[0..2].to_string()),
                        format!("{}.", w[0..3].to_string()),
                    ])
                    .collect::<Vec<_>>()
            ]
        a_day => (?-i) [["M", "T", "W", "R", "F", "S", "U"]]
        a_month => [
                "January February March April May June July August September October November December"
                     .split(" ")
                     .into_iter()
                     .flat_map(|w| vec![w.to_string(), w[0..3].to_string()])
                     .collect::<Vec<_>>()
            ]
        first_time => [[
                "the beginning",
                "the beginning of time",
                "the first moment",
                "the start",
                "the very start",
                "the first instant",
                "the dawn of time",
                "the big bang",
                "the birth of the universe",
            ]]
        last_time => [[
                "the end",
                "the end of time",
                "the very end",
                "the last moment",
                "eternity",
                "infinity",
                "doomsday",
                "the crack of doom",
                "armageddon",
                "ragnarok",
                "the big crunch",
                "the heat death of the universe",
                "doom",
                "death",
                "perdition",
                "the last hurrah",
                "ever after",
                "the last syllable of recorded time",
            ]]
    }.matcher().unwrap();
}

/// A collection of parameters that can influence the interpretation
/// of time expressions.
#[derive(Debug, Clone)]
pub struct Config {
    now: NaiveDateTime,
    monday_starts_week: bool,
    period: Period,
    pay_period_length: u32,
    pay_period_start: Option<NaiveDate>,
}

impl Config {
    /// Constructs an expression with the default parameters.
    pub fn new() -> Config {
        Config {
            now: Local::now().naive_local(),
            monday_starts_week: true,
            period: Period::Minute,
            pay_period_length: 7,
            pay_period_start: None,
        }
    }
    /// Returns a copy of the configuration parameters with the "now" moment
    /// set to the parameter supplied.
    pub fn now(&self, n: NaiveDateTime) -> Config {
        let mut c = self.clone();
        c.now = n;
        c
    }
    fn period(&self, period: Period) -> Config {
        let mut c = self.clone();
        c.period = period;
        c
    }
    /// Returns a copy of the configuration parameters with whether
    /// Monday is regarded as the first day of the week set to the parameter
    /// supplied. By default Monday *is* regarded as the first day. If this
    /// parameter is set to `false`, Sunday will be regarded as the first weekday.
    pub fn monday_starts_week(&self, monday_starts_week: bool) -> Config {
        let mut c = self.clone();
        c.monday_starts_week = monday_starts_week;
        c
    }
    /// Returns a copy of the configuration parameters with the pay period
    /// length in days set to the parameter supplied. The default pay period
    /// length is 7 days.
    pub fn pay_period_length(&self, pay_period_length: u32) -> Config {
        let mut c = self.clone();
        c.pay_period_length = pay_period_length;
        c
    }
    /// Returns a copy of the configuration parameters with the reference start
    /// date for a pay period set to the parameter supplied. By default this date
    /// is undefined. Unless it is defined, expressions containing the phrase "pay period"
    /// or "pp" cannot be interpreted.
    pub fn pay_period_start(&self, pay_period_start: Option<NaiveDate>) -> Config {
        let mut c = self.clone();
        c.pay_period_start = pay_period_start;
        c
    }
}

/// A simple categorization of things that could go wrong.
///
/// Every error provides a descriptive string that can be displayed.
#[derive(Debug, Clone)]
pub enum TimeError {
    /// The time expression cannot be parsed by the available grammar.
    Parse(String),
    /// The time expression consists of a time range and the end of the range is before
    /// the beginning.
    Misordered(String),
    /// The time expression specifies an impossible date, such as the 31st of September.
    ImpossibleDate(String),
    /// The time expression specifies a weekday different from that required by the rest
    /// of the expression, such as Wednesday, May 5, 1969, which was a Tuesday.
    Weekday(String),
    /// The time expression refers to a pay period, but the starting date of a reference
    /// pay period has not been provided, so the pay period is undefined.
    NoPayPeriod(String),
}

/// Converts a time expression into a pair or timestamps and a boolean indicating whether
/// the expression was literally a range, such as "9 to 11", as opposed to "9 AM", say.
///
/// The second parameter is an optional `Config` object. In general you will not need to
/// use this except in testing or in the interpretation of pay periods.
///
/// # Examples
///
/// ```rust
/// # extern crate two_timer;
/// # use two_timer::{parse, Config};
/// # extern crate chrono;
/// # use chrono::{Date, TimeZone, Utc};
/// let (reference_time, _, _) = parse("5/6/69", None).unwrap();
/// ```
pub fn parse(
    phrase: &str,
    config: Option<Config>,
) -> Result<(NaiveDateTime, NaiveDateTime, bool), TimeError> {
    let parse = MATCHER.parse(phrase);
    if parse.is_none() {
        return Err(TimeError::Parse(format!(
            "could not parse \"{}\" as a time expression",
            phrase
        )));
    }
    let parse = parse.unwrap();
    if parse.has("universal") {
        return Ok((first_moment(), last_moment(), false));
    }
    let parse = parse.name("particular").unwrap();
    let config = config.unwrap_or(Config::new());
    if let Some(moment) = parse.name("one_time") {
        return handle_one_time(moment, &config);
    }
    if let Some(two_times) = parse.name("two_times") {
        let first = &two_times.children().unwrap()[0];
        let last = &two_times.children().unwrap()[2];
        let is_through = two_times.has("through");
        if specific(first) {
            if specific(last) {
                return match specific_moment(first, &config) {
                    Ok((d1, _)) => match specific_moment(last, &config) {
                        Ok((d2, d3)) => {
                            let d2 = pick_terminus(d2, d3, is_through);
                            if d1 <= d2 {
                                Ok((d1, d2, true))
                            } else {
                                Err(TimeError::Misordered(format!(
                                    "{} is after {}",
                                    first.as_str(),
                                    last.as_str()
                                )))
                            }
                        }
                        Err(s) => Err(s),
                    },
                    Err(s) => Err(s),
                };
            } else {
                return match specific_moment(first, &config) {
                    Ok((d1, _)) => {
                        let (d2, d3) = relative_moment(last, &config, &d1, false);
                        let d2 = pick_terminus(d2, d3, is_through);
                        Ok((d1, d2, true))
                    }
                    Err(s) => Err(s),
                };
            }
        } else if specific(last) {
            return match specific_moment(last, &config) {
                Ok((d2, d3)) => {
                    let d2 = pick_terminus(d2, d3, is_through);
                    let (d1, _) = relative_moment(first, &config, &d2, true);
                    Ok((d1, d2, true))
                }
                Err(s) => Err(s),
            };
        } else {
            // the first moment is assumed to be before now
            let (d1, _) = relative_moment(first, &config, &config.now, true);
            // the second moment is necessarily after the first momentß
            let (d2, d3) = relative_moment(last, &config, &d1, false);
            let d2 = pick_terminus(d2, d3, is_through);
            return Ok((d1, d2, true));
        }
    }
    unreachable!();
}

// for the end time, if the span is less than a day, use the first, otherwise use the second
// e.g., Monday through Friday at 3 PM should end at 3 PM, but Monday through Friday should end at the end of Friday
fn pick_terminus(d1: NaiveDateTime, d2: NaiveDateTime, through: bool) -> NaiveDateTime {
    if d1.day() == d2.day() && d1.month() == d2.month() && d1.year() == d2.year() {
        d1
    } else if through {
        d2
    } else {
        d1
    }
}

/// The moment regarded as the beginning of time.
///
/// # Examples
///
/// ```rust
/// # extern crate two_timer;
/// # use two_timer::first_moment;
/// println!("{}", first_moment()); // -262144-01-01 00:00:00 UTC
/// ```
pub fn first_moment() -> NaiveDateTime {
    chrono::naive::MIN_DATE.and_hms_milli(0, 0, 0, 0)
}

/// The moment regarded as the end of time.
///
/// # Examples
///
/// ```rust
/// # extern crate two_timer;
/// # use two_timer::last_moment;
/// println!("{}", last_moment()); // +262143-12-31 23:59:59.999 UTC
/// ```
pub fn last_moment() -> NaiveDateTime {
    chrono::naive::MAX_DATE.and_hms_milli(23, 59, 59, 999)
}

fn specific(m: &Match) -> bool {
    m.has("specific_day") || m.has("specific_period") || m.has("specific_time")
}

fn handle_specific_day(
    m: &Match,
    config: &Config,
) -> Result<(NaiveDateTime, NaiveDateTime), TimeError> {
    let now = config.now.clone();
    let mut times = m.all_names("time");
    if times.len() > 1 {
        return Err(TimeError::Parse(format!(
            "more than one daytime specified in {}",
            m.as_str()
        )));
    }
    let time = times.pop();
    if let Some(adverb) = m.name("adverb") {
        return match adverb.as_str().chars().nth(0).expect("empty string") {
            // now
            'n' | 'N' => Ok(moment_and_time(config, time)),
            't' | 'T' => match adverb.as_str().chars().nth(2).expect("impossible string") {
                // today
                'd' | 'D' => Ok(moment_and_time(&config.period(Period::Day), time)),
                // tomorrow
                'm' | 'M' => Ok(moment_and_time(
                    &Config::new()
                        .now(now + Duration::days(1))
                        .period(Period::Day),
                    time,
                )),
                _ => unreachable!(),
            },
            // yesterday
            'y' | 'Y' => Ok(moment_and_time(
                &Config::new()
                    .now(now - Duration::days(1))
                    .period(Period::Day),
                time,
            )),
            _ => unreachable!(),
        };
    }
    if let Some(date) = m.name("date_with_year") {
        if let Some(date) = date.name("n_date") {
            let year = year(date, &now);
            let month = n_month(date);
            let day = n_day(date);
            let d_opt = NaiveDate::from_ymd_opt(year, month, day);
            return match d_opt {
                None => Err(TimeError::ImpossibleDate(format!(
                    "cannot construct UTC date with year {}, month {}, and day {}",
                    year, month, day
                ))),
                Some(d1) => {
                    let d1 = d1.and_hms(0, 0, 0);
                    Ok(moment_and_time(
                        &Config::new().now(d1).period(Period::Day),
                        time,
                    ))
                }
            };
        }
        if let Some(date) = date.name("a_date") {
            let year = year(date, &now);
            let month = a_month(date);
            let day = n_day(date);
            let d_opt = NaiveDate::from_ymd_opt(year, month, day);
            return match d_opt {
                None => Err(TimeError::ImpossibleDate(format!(
                    "cannot construct UTC date with year {}, month {}, and day {}",
                    year, month, day
                ))),
                Some(d1) => {
                    if let Some(wd) = date.name("a_day") {
                        let wd = weekday(wd.as_str());
                        if wd == d1.weekday() {
                            let d1 = d1.and_hms(0, 0, 0);
                            Ok(moment_and_time(
                                &Config::new().now(d1).period(Period::Day),
                                time,
                            ))
                        } else {
                            Err(TimeError::Weekday(format!(
                                "the weekday of year {}, month {}, day {} is not {}",
                                year,
                                month,
                                day,
                                date.name("a_day").unwrap().as_str()
                            )))
                        }
                    } else {
                        let d1 = d1.and_hms(0, 0, 0);
                        Ok(moment_and_time(
                            &Config::new().now(d1).period(Period::Day),
                            time,
                        ))
                    }
                }
            };
        }
        unreachable!();
    }
    unimplemented!();
}

fn handle_specific_period(
    moment: &Match,
    config: &Config,
) -> Result<(NaiveDateTime, NaiveDateTime), TimeError> {
    if let Some(moment) = moment.name("month_and_year") {
        let y = year(moment, &config.now);
        let m = a_month(moment);
        return match NaiveDate::from_ymd_opt(y, m, 1) {
            None => unreachable!(),
            Some(d1) => {
                let d1 = d1.and_hms(0, 0, 0);
                Ok(moment_and_time(
                    &Config::new().now(d1).period(Period::Month),
                    None,
                ))
            }
        };
    }
    if let Some(moment) = moment.name("modified_period") {
        let modifier = PeriodModifier::from_match(moment.name("modifier").unwrap());
        if let Some(month) = moment.name("a_month") {
            let d = config.now.with_month(a_month(month)).unwrap();
            let (d, _) = moment_to_period(d, &Period::Month, config);
            let d = match modifier {
                PeriodModifier::Next => d.with_year(d.year() + 1).unwrap(),
                PeriodModifier::Last => d.with_year(d.year() - 1).unwrap(),
                PeriodModifier::This => d,
            };
            return Ok(moment_to_period(d, &Period::Month, config));
        }
        if let Some(wd) = moment.name("a_day") {
            let wd = weekday(wd.as_str());
            let offset = config.now.weekday().num_days_from_monday() as i64
                - wd.num_days_from_monday() as i64;
            let d = config.now.date() - Duration::days(offset);
            let d = match modifier {
                PeriodModifier::Next => d + Duration::days(7),
                PeriodModifier::Last => d - Duration::days(7),
                PeriodModifier::This => d,
            };
            return Ok(moment_to_period(d.and_hms(0, 0, 0), &Period::Day, config));
        }
        return match ModifiablePeriod::from_match(moment.name("modifiable_period").unwrap()) {
            ModifiablePeriod::Week => {
                let (d, _) = moment_to_period(config.now, &Period::Week, config);
                let d = match modifier {
                    PeriodModifier::Next => d + Duration::days(7),
                    PeriodModifier::Last => d - Duration::days(7),
                    PeriodModifier::This => d,
                };
                Ok(moment_to_period(d, &Period::Week, config))
            }
            ModifiablePeriod::Weekend => {
                let (_, d2) =
                    moment_to_period(config.now, &Period::Week, &config.monday_starts_week(true));
                let d2 = match modifier {
                    PeriodModifier::Next => d2 + Duration::days(7),
                    PeriodModifier::Last => d2 - Duration::days(7),
                    PeriodModifier::This => d2,
                };
                let d1 = d2 - Duration::days(2);
                Ok((d1, d2))
            }
            ModifiablePeriod::Month => {
                let (d, _) = moment_to_period(config.now, &Period::Month, config);
                let d = match modifier {
                    PeriodModifier::Next => {
                        if d.month() == 12 {
                            d.with_year(d.year() + 1).unwrap().with_month(1).unwrap()
                        } else {
                            d.with_month(d.month() + 1).unwrap()
                        }
                    }
                    PeriodModifier::Last => {
                        if d.month() == 1 {
                            d.with_year(d.year() - 1).unwrap().with_month(12).unwrap()
                        } else {
                            d.with_month(d.month() - 1).unwrap()
                        }
                    }
                    PeriodModifier::This => d,
                };
                Ok(moment_to_period(d, &Period::Month, config))
            }
            ModifiablePeriod::Year => {
                let (d, _) = moment_to_period(config.now, &Period::Year, config);
                let d = match modifier {
                    PeriodModifier::Next => d.with_year(d.year() + 1).unwrap(),
                    PeriodModifier::Last => d.with_year(d.year() - 1).unwrap(),
                    PeriodModifier::This => d,
                };
                Ok(moment_to_period(d, &Period::Year, config))
            }
            ModifiablePeriod::PayPeriod => {
                if config.pay_period_start.is_some() {
                    let (d, _) = moment_to_period(config.now, &Period::PayPeriod, config);
                    let d = match modifier {
                        PeriodModifier::Next => d + Duration::days(config.pay_period_length as i64),
                        PeriodModifier::Last => d - Duration::days(config.pay_period_length as i64),
                        PeriodModifier::This => d,
                    };
                    Ok(moment_to_period(d, &Period::PayPeriod, config))
                } else {
                    Err(TimeError::NoPayPeriod(String::from(
                        "no pay period start date provided",
                    )))
                }
            }
        };
    }
    unreachable!()
}

enum ModifiablePeriod {
    Week,
    Month,
    Year,
    PayPeriod,
    Weekend,
}

impl ModifiablePeriod {
    fn from_match(m: &Match) -> ModifiablePeriod {
        match m.as_str().chars().nth(0).expect("unreachable") {
            'w' | 'W' => {
                if m.as_str().len() == 4 {
                    ModifiablePeriod::Week
                } else {
                    ModifiablePeriod::Weekend
                }
            }
            'm' | 'M' => ModifiablePeriod::Month,
            'y' | 'Y' => ModifiablePeriod::Year,
            'p' | 'P' => ModifiablePeriod::PayPeriod,
            _ => unreachable!(),
        }
    }
}

enum PeriodModifier {
    This,
    Next,
    Last,
}

impl PeriodModifier {
    fn from_match(m: &Match) -> PeriodModifier {
        match m.as_str().chars().nth(0).expect("unreachable") {
            't' | 'T' => PeriodModifier::This,
            'l' | 'L' => PeriodModifier::Last,
            'n' | 'N' => PeriodModifier::Next,
            _ => unreachable!(),
        }
    }
}

fn handle_specific_time(
    moment: &Match,
    config: &Config,
) -> Result<(NaiveDateTime, NaiveDateTime), TimeError> {
    return if moment.has("first_time") {
        Ok(moment_to_period(first_moment(), &config.period, config))
    } else {
        Ok((last_moment(), last_moment()))
    };
}

fn handle_one_time(
    moment: &Match,
    config: &Config,
) -> Result<(NaiveDateTime, NaiveDateTime, bool), TimeError> {
    let r = if moment.has("specific_day") {
        handle_specific_day(moment, config)
    } else if let Some(moment) = moment.name("specific_period") {
        handle_specific_period(moment, config)
    } else if let Some(moment) = moment.name("specific_time") {
        handle_specific_time(moment, config)
    } else {
        Ok(relative_moment(moment, config, &config.now, true))
    };
    match r {
        Ok((d1, d2)) => Ok((d1, d2, false)),
        Err(e) => Err(e),
    }
}

// add time to a date
fn moment_and_time(config: &Config, daytime: Option<&Match>) -> (NaiveDateTime, NaiveDateTime) {
    if let Some(daytime) = daytime {
        let (hour, minute, second) = time(daytime);
        let period = if second.is_some() {
            Period::Second
        } else if minute.is_some() {
            Period::Minute
        } else {
            Period::Hour
        };
        let m = config
            .now
            .with_hour(hour)
            .unwrap()
            .with_minute(minute.unwrap_or(0))
            .unwrap()
            .with_second(second.unwrap_or(0))
            .unwrap();
        moment_to_period(m, &period, config)
    } else {
        moment_to_period(config.now, &config.period, config)
    }
}

fn relative_moment(
    m: &Match,
    config: &Config,
    other_time: &NaiveDateTime,
    before: bool,
) -> (NaiveDateTime, NaiveDateTime) {
    if let Some(day) = m.name("a_day") {
        let wd = weekday(day.as_str());
        let mut delta =
            other_time.weekday().num_days_from_sunday() as i64 - wd.num_days_from_sunday() as i64;
        if delta <= 0 {
            delta += 7;
        }
        let mut d = other_time.date() - Duration::days(delta);
        if !before {
            d = d + Duration::days(7);
        }
        return moment_and_time(
            &config.now(d.and_hms(0, 0, 0)).period(Period::Day),
            m.name("time"),
        );
    }
    if let Some(t) = m.name("time") {
        let (hour, minute, second) = time(t);
        let period = if second.is_some() {
            Period::Second
        } else if minute.is_some() {
            Period::Minute
        } else {
            Period::Hour
        };
        let mut t = other_time
            .with_hour(hour)
            .unwrap()
            .with_minute(minute.unwrap_or(0))
            .unwrap()
            .with_second(second.unwrap_or(0))
            .unwrap();
        if before && t > *other_time {
            t = t - Duration::days(1);
        } else if !before && t < *other_time {
            t = t + Duration::days(1);
        }
        return moment_to_period(t, &period, config);
    }
    if let Some(month) = m.name("a_month") {
        let month = a_month(month);
        let year = if before {
            if month > other_time.month() {
                other_time.year() - 1
            } else {
                other_time.year()
            }
        } else {
            if month < other_time.month() {
                other_time.year() + 1
            } else {
                other_time.year()
            }
        };
        let d = NaiveDate::from_ymd(year, month, 1).and_hms(0, 0, 0);
        let (d1, d2) = moment_to_period(d, &Period::Month, config);
        if before && d1 >= *other_time {
            return moment_to_period(d1.with_year(d1.year() - 1).unwrap(), &Period::Month, config);
        } else if !before && d2 <= *other_time {
            return moment_to_period(d1.with_year(d1.year() + 1).unwrap(), &Period::Month, config);
        }
        return (d1, d2);
    }
    unreachable!()
}

fn specific_moment(
    m: &Match,
    config: &Config,
) -> Result<(NaiveDateTime, NaiveDateTime), TimeError> {
    if let Some(m) = m.name("specific_day") {
        return handle_specific_day(m, config);
    }
    if let Some(m) = m.name("specific_period") {
        return handle_specific_period(m, config);
    }
    if let Some(m) = m.name("specific_time") {
        return handle_specific_time(m, config);
    }
    unreachable!()
}

fn a_month(m: &Match) -> u32 {
    match m.name("a_month").unwrap().as_str()[0..3]
        .to_lowercase()
        .as_ref()
    {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => unreachable!(),
    }
}

// extract hour, minute, and second from time match
fn time(m: &Match) -> (u32, Option<u32>, Option<u32>) {
    let hour = if let Some(hour_24) = m.name("hour_24") {
        s_to_n(hour_24.name("h24").unwrap().as_str())
    } else if let Some(hour_12) = m.name("hour_12") {
        let hour = s_to_n(hour_12.name("h12").unwrap().as_str());
        if let Some(am_pm) = m.name("am_pm") {
            match am_pm.as_str().chars().nth(0).expect("empty string") {
                'a' | 'A' => hour,
                _ => hour + 12,
            }
        } else {
            hour
        }
    } else {
        unreachable!()
    };
    if let Some(minute) = m.name("minute") {
        let minute = s_to_n(minute.as_str());
        if let Some(second) = m.name("second") {
            let second = s_to_n(second.as_str());
            (hour, Some(minute), Some(second))
        } else {
            (hour, Some(minute), None)
        }
    } else {
        (hour, None, None)
    }
}

fn n_month(m: &Match) -> u32 {
    lazy_static! {
        static ref MONTH: Regex = Regex::new(r"\A0?(\d{1,2})\z").unwrap();
    }
    let cap = MONTH.captures(m.name("n_month").unwrap().as_str()).unwrap();
    cap[1].parse::<u32>().unwrap()
}

fn year(m: &Match, now: &NaiveDateTime) -> i32 {
    let year = m.name("year").unwrap();
    if let Some(sy) = year.name("short_year") {
        let y = s_to_n(sy.as_str()) as i32;
        let this_year = now.year() % 100;
        if this_year < y {
            now.year() - this_year - 100 + y
        } else {
            now.year() - this_year + y
        }
    } else if let Some(suffix) = year.name("year_suffix") {
        let y = s_to_n(year.name("suffix_year").unwrap().as_str()) as i32;
        if suffix.has("bce") {
            1 - y // there is no year 0
        } else {
            y
        }
    } else {
        let y = s_to_n(year.name("n_year").unwrap().as_str()) as i32;
        if year.as_str().chars().nth(0).expect("unreachable") == '-' {
            -y
        } else {
            y
        }
    }
}

fn s_to_n(s: &str) -> u32 {
    lazy_static! {
        static ref S_TO_N: Regex = Regex::new(r"\A[\D0]*(\d+)\z").unwrap();
    }
    S_TO_N.captures(s).unwrap()[1].parse::<u32>().unwrap()
}

fn n_day(m: &Match) -> u32 {
    m.name("n_day").unwrap().as_str().parse::<u32>().unwrap()
}

/// expand a moment to the period containing it
fn moment_to_period(
    now: NaiveDateTime,
    period: &Period,
    config: &Config,
) -> (NaiveDateTime, NaiveDateTime) {
    match period {
        Period::Year => {
            let d1 = NaiveDate::from_ymd(now.year(), 1, 1).and_hms(0, 0, 0);
            let d2 = NaiveDate::from_ymd(now.year() + 1, 1, 1).and_hms(0, 0, 0);
            (d1, d2)
        }
        Period::Month => {
            let d1 = NaiveDate::from_ymd(now.year(), now.month(), 1).and_hms(0, 0, 0);
            let d2 = if now.month() == 12 {
                NaiveDate::from_ymd(now.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd(now.year(), now.month() + 1, 1)
            }
            .and_hms(0, 0, 0);
            (d1, d2)
        }
        Period::Week => {
            let offset = if config.monday_starts_week {
                now.weekday().num_days_from_monday()
            } else {
                now.weekday().num_days_from_sunday()
            };
            let d1 = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
                - Duration::days(offset as i64);
            (d1, d1 + Duration::days(7))
        }
        Period::Day => {
            let d1 = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0);
            (d1, d1 + Duration::days(1))
        }
        Period::Hour => {
            let d1 =
                NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(now.hour(), 0, 0);
            (d1, d1 + Duration::hours(1))
        }
        Period::Minute => {
            let d1 = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(
                now.hour(),
                now.minute(),
                0,
            );
            (d1, d1 + Duration::minutes(1))
        }
        Period::Second => {
            let d1 = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(
                now.hour(),
                now.minute(),
                now.second(),
            );
            (d1, d1 + Duration::seconds(1))
        }
        Period::PayPeriod => {
            if let Some(pps) = config.pay_period_start {
                // find the current pay period start
                let offset = now.num_days_from_ce() - pps.num_days_from_ce();
                let ppl = config.pay_period_length as i32;
                let mut offset = (offset % ppl) as i64;
                if offset < 0 {
                    offset += config.pay_period_length as i64;
                }
                let d1 = (now.date() - Duration::days(offset)).and_hms(0, 0, 0);
                (d1, d1 + Duration::days(config.pay_period_length as i64))
            } else {
                unreachable!()
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Period {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    PayPeriod,
}

fn weekday(s: &str) -> Weekday {
    match s.chars().nth(0).expect("empty string") {
        'm' | 'M' => Weekday::Mon,
        't' | 'T' => {
            if s.len() == 1 {
                Weekday::Tue
            } else {
                match s.chars().nth(1).unwrap() {
                    'u' | 'U' => Weekday::Tue,
                    'h' | 'H' => Weekday::Thu,
                    _ => unreachable!(),
                }
            }
        }
        'w' | 'W' => Weekday::Wed,
        'H' => Weekday::Thu,
        'F' | 'f' => Weekday::Fri,
        'S' | 's' => {
            if s.len() == 1 {
                Weekday::Sat
            } else {
                match s.chars().nth(1).unwrap() {
                    'a' | 'A' => Weekday::Sat,
                    'u' | 'U' => Weekday::Sun,
                    _ => unreachable!(),
                }
            }
        }
        'U' => Weekday::Sun,
        _ => unreachable!(),
    }
}
