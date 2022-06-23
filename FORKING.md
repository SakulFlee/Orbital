# How to properly 'fork' this template/example?

We have a [base template/example](https://github.com/rust-multiplatform/Base-Project-Template).  
In it, we defined the basics of our projects to keep them uniform across all other examples/templates, as well as developer tools like CI's (Continuous Integrations).

When using this example/template, we want to ensure that everyone is up-to-date and secure.
This is commonly done with **forks**.  
However, there is a problem: Within the same organization or namespace (e.g. user) you can't fork a repository when it already exists.
This means we would have to create a separate namespace for each example we provide.
However, there is a way around this issue.

## Normal forking (works just once per namespace)

On GitHub on our [base template/example](https://github.com/rust-multiplatform/Base-Project-Template) at the top right corner is a "Fork" button.
Simply press that, give your new repository a name and select the namespace and done.  
You've just forked our repository!
Every time we update the base repository, you will find a banner and button on your forked repository stating that the base got updated and that you can merge it into your fork.
**To stay up to date and avoid future problems we recommend to always merge!**

## Native forking (works unlimited)

However, when working on a project internally (in the same namespace/organization) we can't easily fork as the project already exists there.  
Weird limitation, but we can get around it:

> Note that this also works outside of GitHub.  
> This is possible for every Git repository, no matter where it is hosted.  
> Even local repositories can use this.

First, we need to create a full copy (mirror) of the base:  

1. Create a new empty repository in the namespace you want it to.
2. Clone the [base template/example](https://github.com/rust-multiplatform/Base-Project-Template) in **mirror** mode: `git clone --mirror https://github.com/rust-multiplatform/Base-Project-Template Base-Project-Template`
3. Go into your project folder and push to your project in **mirror** mode: `git push --mirror <your git url>`

Next, we modify the new repository to work like "forks":  

> Note that if you already have a repository setup (mirrored) and are just cloning your repository on a new machine or something similar: these following steps have to be repeated for forking to properly work!  
> Although, having this on ONE machine (e.g. your head-developer) is enough.
> Not every member using the repository must do this.

1. Clone your repository normally (`git clone <your git url> <folder name>`)
2. Add a secondary remote commonly called `upstream` with the base project url: `git remote add upstream https://github.com/rust-multiplatform/Base-Project-Template`
3. Now, using `git fetch --all` (or `git fetch upstream`) will update your mirror repository with the base projects changes.
4. With `git pull upstream (<branch>)` you can pull the changes from the base repository. This will automatically start a merge and may prompt you with merge conflicts if you've changed any of the files. Resolve those, commit the merge and you are done updating your fork/mirror!

Unfortunately, this is harder to automate as merge conflicts usually require user interaction.  
Furthermore, there is currently no way of checking (or getting a notification) _if_ the base updated, except for manually checking and/or pulling from upstream.

### Automation

To automate this process we could make a CI/GitHub Action that runs hourly or daily (or whenever) (note that on-push won't work since changes are happening on the base repository, not yours!) and does the latter steps automatically.  
However, this will fail if even a single merge conflict arises in the forked repository.  
This then would have to be solved manually.
