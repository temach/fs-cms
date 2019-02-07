# fs-cms
Content management system for websites based on filesystem structure

# Development roadmap

-   Detect artifacts and templates

-   Embed simple artifacts into templates. Start with text. If you can embed text, 
    then everything else can be done via links, unless you are hosting the content.

-   Arrangement of artifacts with respect to each other.

-   Recursive artifacts. Evaluate a folder and produce an artifact.

-   Embed complex artifacts that are self-hosted: images, videos.

-   Embed artifacts that are hosted on other sites: link to soundcloud becomes
    embedded soundcloud player, link to googlemaps becomes google-map embedded on page.
    Maybe this is a pointless idea actually.

-   Use CSS grid to specify layout of artifacts on page? See: https://gridbyexample.com/examples/
    Should provide a couple of templates. Grid intro: https://www.smashingmagazine.com/2018/05/future-of-web-design/

-   Should geenrate CSS classes from file names. See how SmallVistories does this. 
    For file my_photo_important.png they would split on "_" and generate 3 css classes:
    .my, .photo, .important which you can override in a style sheet.

-   The layout templates are .css files, they should be made somehow viewable in a
    browser. Then the user will have an easier time selecting and modifying.

-   Ability to drop in Javascript and CSS files? Maybe have one JS/CSS file and everything
    should be copy-pasted there?

-   Ignore artifacts starting with an underscore or maybe its better to use underscore for system files?

-   Great place to layout grid: https://www.layoutit.com/grid
    Also might consider: https://github.com/sylvainpolletvillard/postcss-grid-kiss
    The guy has the right idea that css grid currently is easier to write than to read and he
    wants to reverse this. This is exactly my use case: users need to be able to read css
    grid declaration and choose from templates! They are not expected to actually write them.
