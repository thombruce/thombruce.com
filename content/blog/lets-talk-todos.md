---
title: Let's Talk Todos
date: 2021-10-30T18:26:10Z
---

Okay, following on from my last blog post I now have just the one website hosted on Netlify - this one. A few others are hosted on GitHub pages, but my intention is to leave them that way... with the possible exception of my screenplays website, which lists the screenplays I wrote years ago. I'm half wondering whether I should move those screenplays over here, into the thombruce.com repo for presentation on this website at, say, thombruce.com/screenplays... but I don't know. There's honestly no reason to have them online right now, anyway. I'm not pointing anybody to them; I'm not discussing them, apart from here where I'm just discussing whether they should be here, there or online at all... But I do want to get back into writing. I'll leave them where they are for now (https://thombruce.github.io/screenplays/) and maybe address that question in the future when I... make a little progress with these other todos...

So what did I mention last time? Several apps I'd like to develop and manage? A lot of fiction I'd like to write? Some video games I'd like to produce..? I think it's worth my while to detail those plans a little more. For example, "produce video games" is a significant overselling of my intentions there just now; I just want to code some simple games, inspired by classics like Pong and Asteroids as an exercise. Nothing more than that for a start. _If only my fiction writing ambitions could start so simply._ Anyways, let's get into the what and the why...

## Apps & Software

### TNT

We'll start with an ongoing project that I'm currently using. In fact, ThomBruce.com is making liberal use of this project. TNT, which I think needs a better name, is a style and component library for Vue.js and Nuxt.js. It's based on [Tailwind](https://tailwindcss.com/) and [DaisyUI](https://daisyui.com/). Additionally, it includes Pug, Font Awesome and Lodash. The Nuxt version also comes with Nuxt Image and my own Nuxt Content extension, Nuxt Taxonomies. Actually, perhaps I should list Nuxt Taxonomies here too... Okay, yeah, next entry. The point of TNT anyways, is to bring together common tools that I use for many projects so that I don't have to bother with multiple installations and configurations; TNT installs them all and should come preconfigured for most of my needs. That's the plan, and it's almost there. I don't intend to have an official release prepared before Nuxt 3 has matured and has support for the Nuxt Content module, which is a while off. As of now, TNT is in Alpha and loosely works with Vue 2 and Nuxt 2.

### Nuxt Taxonomies

As mentioned a moment ago, Nuxt Taxonomies is a Nuxt Content extension. Essentially, it's a convenience plugin with an opinionated approach to extracting and listing "taxonomies" as included in the meta headers of Nuxt content documents. So like tags, categories, series, but it works for any array-based attribute and can be employed either globally or nested within a content section. It's a little more ready for use than TNT, so let's call this a Beta. I might try to maintain support for Nuxt 2 on this one, but like TNT it will likely be revamped for Nuxt 3. We anticipate the Nuxt Content module becoming compatible with the new version of Nuxt.

### ~~Marmalade~~

~~All right, here's some projects I've developed in the past that made it pretty- ... Actually, let's switch up the order.~~

### Quotable

The first of these actually is Quotable. It's the first project I developed when I got serious about programming. It's the reason I'm a Ruby on Rails developer rather than... .NET or Python or whatever. Rails made the most sense when I chose it because it had a strong set of natural language processing and HTML parsing features and plugins. What Quotable did with those was it parsed HTML documents, separating each paragraph into its constituent sentences (harder than it sounds) so that they could then be individually selected with a single click, or a tap on mobile phone. I still think there's strength in this idea, and I want to revisit it... but not in Ruby. It makes a lot of sense on mobile, but I might be being too hopeful if I expect JavaScript will have all of the features I needed by now. I'm gonna give it a shot in JS, nonetheless, but we may find it needs to be developed in another language.

But what is the actual point of Quotable? Well, originally for lack of a better idea it was... a social media platform. I could still do... _something_ like that. But I do see more broad applications for the core code. I'll probably develop a plugin that I can use on other websites, including blogs like this one.

### Marmalade

Okay, so Marmalade... Marmalade _was_ a personal and business organisation/management platform. It had tools for reviewing your finances, managing projects, taking stock of inventory, and determining the worth of a business and of your stake in it, as well as household finances, todos, etc. Ambitious, then? Yes. But it fundamentally worked; I had a minimum viable product, and those features were all well-integrated. Changes to one financial account in, say, an independent business echoed up to your personal finances, so you could see at a glance what your personal takings were or might be and, if combined with other household members, how much expendable income your household had. The unique selling point here was connectivity; I personally still believe a product like this could appeal to independent business owners. And I did pitch it to a group online who... for the most part... said it did too much. There is a difference, they said, between a tool and a platform, and most users prefer tools that do one thing really well, rather than a platform that does a lot poorly. I get that, I do. And yet I still think this thing has legs.

It needs to be redeveloped for modern web standards, so I'm starting from scratch anyways. And what I'm thinking, based on that original feedback, is that I should approach this like a suite of tools, not like a single platform. So we might have... Marmalade Todos, Marmalade Home, Marmalade Business. Behind the scenes, however, I still want this all integrated such that todo or project costs can be viewed in the financial side of the application, and such that inventory value and purchases can be associated with finances and projects too, and such that independent business owners can see the worth of their business in personal and household terms. It's difficult to imagine the individual tools working apart from the whole, but if this is purely something for myself at the end of the day, so be it. I can make use of this thing. I want this thing, and I don't think I'm unique in that respect.

### Credible... maybe

Lastly in terms of projects I've developed before, Credible was an alternative Ruby on Rails authentication solution to the existing solutions like... Devise. Mainly Devise. See, whereas Devise provides an excellent cookie-based authentication solution, it's difficult to get working with app architecture on a separate domain; it's difficult to get working with JSON Web Tokens. But Devise is also just a wrapper around Warden. Warden is the core of Devise and what provides the security to Devise's authentication solution. Thus, I found it easier to make my own wrapper around Warden, that supported JWTs, than to fiddle with Devise.

I will probably be taking this route again as I develop Marmalade and, as yet unmentioned, Inkpot and Hosted. So I am very likely to recreate the Credible gem. But I'm talking platform here. Because there isn't a lot of difference between Credible, as a Ruby gem, and the Auth0 authentication as a service platform. Not really. If you slap a UI around the Credible gem, then hey presto, you've basically got an authentication as a service platform; such is the design of JSON Web Tokens, anyways. So this is something I want to consider, at least. An open-source, self-hosted authentication platform.

Now, fundamentally I see Marmalade, Inkpot and Hosted having their own authentication solutions based on Credible (and using the gem), but... ThomBruce.com is a static website; I could in theory allow people to log in here with an external solution. There's utility in a self-hosted authentication as a service platform, if I can find it. If I don't, then I probably won't get around to developing this... but hey, it's listed for posterity.

### Inkpot

Inkpot's pretty simple... to explain, at least, certainly not to develop. It's a writing platform. That's it. I envision it as a platform having not just a writing tool, but also planning and research tools, as well as spin-offs for scriptwriting and academic writing. Possibly also a blogging platform..? Though I don't know if Hosted, below, is a better fit for that? Or... maybe Hosted uses Inkpot? Clearly still some things to work out there, but the concept is simple enough.

### Hosted

Hosted, then, is a suite of online management tools... meaning I see it as a single source for managing hosting, domains, maybe content (as noted above) and maybe email (but this is dangerous and tricky territory I'd like to avoid diving into). There's nothing particularly novel about this. But at the moment I'm hosting several sites on Netlify and GitHub and managing their domains from Clouflare, even though they were purchased from Namecheap. In other words, I have several dashboards I have to check every time I want to make some fairly simple changes. So I don't see myself getting into hosting any domain name servers - I wouldn't even know how. Instead, Hosted will connect with those services' APIs so that I can integrate them into the one dashboard. Problem...?

Yeah, problem; not everybody uses those services. This is something being built to my specific needs? It certainly seems that way, and I don't want that to be the case. Whatever the platform can do _by itself_ it will do. Whatever it requires a third party platform to achieve (domains, hosting, etc.), ideally we keep it modular... we support options, plugins that enable the community to develop their own quick solutions which integrate with the platform. In other words... my version can depend on Namecheap, GitHub and Netlify, but others can forego those plugins in favour of others. So we keep it modular, to an extent. We'll cross that bridge when we get there...

### Just for fun...

Finally in terms of apps and software, there's a ton of other stuff I have done or want to do but isn't worth going into detail over. Some of these are just exercises, others are simple widgets and some are simple game concepts I want to develop in a hobbyist capacity to drill new skills. An incomplete list might be...

- Calculator
- FizzBuzz
- Timepiece
- Really Simple Tennis Game
- Really Simple Space Game
- Really Simple Earth Defender Game
- Really Simple Serpent Game

## Fiction

Right then, in terms of fiction... Funny story: I have been procrastinating for TEN YEARS! In fact, Quotable started as a platform for sharing writing... and now I want to develop Inkpot as the tool I would use to write my fiction. Some of the things here, I have wanted to write for a very long time. I need to have a chat with myself about wanting to write and forever procrastinating by coding instead, really. But anyways, let's talk some of these projects and progress that may or may not have been made so far...

### Verse

Verse is by far the project I am most passionate about right now. I've drafted various notes over the years, some of which are now outdated or redundant. The challenge here is that I intend it to be hard science fiction... very, very hard science fiction. That is to say that, unless I have a solid justification or need to do otherwise, the entire narrative should be as scientifically accurate as possible... and oh, boy, the intended narrative is vast. I really wish to describe the entire universe from the moments of the Big Bang, up to the emergence of humanity and our first ventures into the solar system, connecting these events and our evolution thematically and, hopefully, with emotional resonance. An ambitious task but, according to my notes, absolutely possible. Actually, here's a brief excerpt from a recent note that tries to plan the structure of not necessarily just one novel but the overarching structure to an extent...

> Act I. Genesis
>
> Theme: Entropy increases disorder; Conclusion: Life emerges from entropy, creates ordered forms out of chaos.
>
> Act II. Life
> 
> Theme: Competition for energy, size dominates; Conclusion: Humankind invents tools, adjusts the balance.
> 
> Act III. Humanity
> 
> Theme: Invention/Exploration/Consumption of Resources; Conclusion: Environmental disaster.
> 
> Act IV. Machinery (?)
> 
> Theme: Industrialisation has led to environmental crisis; Conclusion: Smarter, more precise industry could turn back the tide (+ artificially sentient robots???)
> 
> Act V. Contact (?)
> 
> Theme: Humanity and machines colonise solar system for the first time in earnest, use knowledge of galaxy to refine energy use; Conclusion: First contact/observation with extra-solar species (?)

...so we are, as you can see, thinking about near future developments as well as... ancient and prehistory. Note the inversion of the theme in each act; how apparently ordered organisms are formed from laws of increasing disorder in the first, how competition is driven by biology up until early hominids develop tools which change the rules... A lot more form is needed, and I do have more detailed notes about the future which... like I said, may be redundant, especially if I want to go really hard sci-fi... I just love this concept.

### Warp

If Verse is super serious sci-fi, Warp is my excuse to fling characters into the distant future and get a little silly with it. Fewer scientific constraints, more grand space opera.

### Mist

Mist... is a fantasy series I started writing in earnest last year. I didn't love the characters I was writing, however, so abandoned that particular arc. Interestingly - _at least to me_ - scientific accuracy is also at the heart of this one. I want to explore a fantasy world, with magic, fantasy creatures and races, but do so through the lens of physics, chemistry and biology. I don't necessarily intend to explain it in text (the setting is, like a lot of fantasy, historic), but I want to be able to personally justify not just the existence of this world but also what the characters do understand of it. I envision the core setting as being during an age of enlightenment, so while their science will not be complete it should be based on science that can be explained... to an extent. I mean, I listed magic as one of the elements; I don't know how based in real science that will be. My early conceptions of the magic in this world are exaggerations of general relativity and quantum mechanics. For example... a region of space where the fundamental laws are just different enough to bend gravity and light in such a way as the planet might appear turned inside out; or in terms of exaggerated quantum mechanics, imagine a mountain range that has no given location but moves when unobserved. Impossibilities in reality, but loosely justified scientifically... emphasis on the **loosely**, not at all on the _scientifically_.

### The Boltzmann God

Bad title? Yeah, it's not got one yet. This story would be a one off based on a script I wrote called _Last Thursday_. Actually, six scripts (it was a miniseries). I want to change it up drastically, but the core of the concept is the same. The protagonist is, in a sense, God. But he doesn't know it. And there's ambiguity as to whether the reality he inhabits is real or imagined. It's possible it's both.

I started writing this one in 2011, when part of the justification for the character's confusion/awakening was drug use. Not a direction I want to go in with it any longer; I have other ideas now. It's always been a tricky one for me to write properly. Ten years of thinking about it, and I still haven't got a full grasp on precisely what I want to do with it.

Probably about time I re-read those screenplays...

### Prose adaptations of my existing screenplays

Speaking of which, I've got other screenplays... _A Silent Musical_, _To Be Wed_, _06665 387729_ (bad title). I've though about turning those three into short stories or novels too. In particular, I adore the story I wrote for _A Silent Musical_ and I think it's wasted in an unproduced script. The concept does benefit from a visual medium, I think, but I think I could adapt it for a piece of prose... maybe novella length, maybe a little longer. The other two would be shorts, but I think adapting them would be a good writing exercise for me.

## End

That's it. That's everything. Go away now. I had more I wanted to list, but I've second-guessed myself. This is already getting on in terms of length, so I'm calling it. Fin.
