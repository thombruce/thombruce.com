---
title: On Unlabel and Marmalade
date: 2021-11-25T10:36:02Z
---

I want to create a todo list app, and I want to create a calendar app.

Problem: These sound like they should be separate but some todos do come attached to deadlines or starting dates... They should show on a calendar.

I also want to create a finance management app... and some events do have an associated cost - whether that be the price of a ticket, a meal or materials.

So I have this broader idea of bringing together todos with a calendar and with finances... and with a bunch of other things... because it makes so much sense to me to integrate these concepts and measure the consequences of one against the other. Like... _oh, we have this birthday meal planned for two weeks from now; here's a todo to purchase a gift and here's the estimated costs of a gift and the meal so we can see how that effects our budget for the month_.

This, actually, was the principle at the core of an app I developed years ago called _Marmalade_. On pitching that, I was told two things...

1. It does too much; customers prefer apps that do a specific thing well over one that does many.
2. The USP (Unique Selling Point) really isn't clear.

Now... those two things can be solved together pretty simply, by doing away with some of the features, focusing in on one or two and really selling the integration between those as a USP. That's... oversimplifying things, but it works.

But I still really want this app that integrates... everything. I still really want to see my original vision for _Marmalade_ through.

Now, I mentioned _Unlabel_ in my previous post here. I said I want to do a bunch of white label products, and... err... I do. I want to do a white label todo app and calendar app, and a white label finances app. And I ultimately do not want to be maintaining many, many very similar things. In other words... _Unlabel_ and _Marmalade_ should be related. Probably: _Marmalade_ should make use of the _Unlabel_ app architecture. This gets me thinking...

Is there a way I can have my cake here, and also eat it? Can I do singular, focused white label apps that also combine together in some modular way to meet the vision I originally had for _Marmalade_?

Well... To know if that's possible, we first really have to think about microservice and database architecture. I'm talking... say we make a todo app; that app needs user accounts, lists and todos stored in its database. Then, say we make a calendar app; that again needs user accounts, and then events with specific date attributes. These apps will all have user accounts in common as one requirement of the database, and then they will have divergent data needs from there (some needs will converge, which is a whole other headache but let's ignore that for now)...

So, first up... even though I want these _Unlabel_ apps to be deployable in isolation with their own user accounts systems... it also needs to be possible to share user accounts between them. This could mean using one application to authenticate the other (an oAuth like pattern), or it could mean they have a shared user system... but this, at a glance, appears to violate the 'deployable in isolation' requirement.

After that, we need to consider their divergent needs and how those integrate. Each app maintaining its own records for todos or calendar events, that's easy. But tying todos to events, and vice versa... That's a little more complicated if they're, y'know, separate applications. We need to communicated via API... which behind the scenes means hitting up two databases with similar queries. At that point, we hit a comparison problem of sorts. Say the todos app has MANY records attached to events, and the calendar app has only few independently created events of its own. When we query these within a specific date range, we may find ourselves wanting to recombine the data... but there are more todos for a given range than events. We have to be aware that we need to query the todos api several more times to be sure where the events fit in the ordered list. _Okay, that didn't make sense - people familiar with this problem, though, will know what I'm talking about. Say you pull events for the range between the current Friday and previous Monday and get two events - one on Friday, one on Tuesday - and your query for todos returns those for Friday, Thursday and Wednesday then stops due to a record limit. You do not know whether there are more todos on Wednesday and Tuesday until you fetch the next page, so you don't yet know where to place the event you have for Tuesday in the ordered list._ We can address that programmatically, I'm just not convinced that it's the best approach...

When I was making _Marmalade_ originally, I had a plan to create smaller, more focused applications based on the same large, central database. This way, specific queries for divergent information all hit the same database. The queries (written in SQL) got kind of complicated, but the data was already integrated; no difficult programmatic logic in the interface between user and data - it all happens in the one app making that data request.

Okay, so that's the problem. _Marmalade_ and _Unlabel_ kind of want to take different approaches to the same application solutions. I don't want to maintain both, and I think that's reasonable - I think I absolutely shouldn't even think about maintaining two wildly different solutions. So we've gotta figure out just... what is _Unlabel_ and how **should** _Marmalade_ really work?

- Unlabel is a series of white label applications for use by developers and business owners to offer as solutions to their customers
- Marmalade **should**...

That's the hard part.

One major feature of _Marmalade_ integrated late into its development was a graph database which didn't replace the relational database I was already using, but enhanced data requests. For complex purposes, the graph database could be queried first to bring up a list of distantly related nodes; these node IDs could then be looked up in the relational database. What this achieved was to replace an N+1 data query on the relational database (meaning an infinite series of queries that would cause slowdown or a crash) with just two or three database hits - one for the initial record, one to fetch relationships from the graph, and one to get the records from the relational database.

In other words... _Marmalade_ had sort of begun to adopt a microservice approach. Don't get me wrong, it was nowhere close to being a microservice-based architecture, but it had already begun to separate out the scopes of query logic, and to great advantage.

So... question: **Could** _Marmalade_ be a collection of microservices (one for events, one for checklist items like todos, one for finances, etc.) that couple together via a separate graph database that describes data relationships.

Let's consider what that would look like...

1. We fetch whatever current record we're interested in.
2. We look up its relations in the graph database.
3. Using information from the graph, we query... as many other microservices as are relevant to present a complete set of records.

Not... great. Suddenly we've reintroduced the N+1 problem. We now have no idea how many API queries we'll be making. Now, we could work around that by storing commonly required information in the Graph database, or in a separate universal cache. We only really need to hit up those specific other microservices to when we either want to inspect their data more closely or update them. That's... _another_ problem.

Updating records will mean hitting the specific microservice that is relevant, which should then communicate with the graph to see if it's relevant to update other records. For instance, this was part of _Marmalade_'s finances architecture - one update to, say, the sale of an item, echoed up the relationship tree and changed company assets, worth of shares, individual net worths, etc. etc. This can mean many insertions, which is again an N+1 problem. It... in that very specific case, _can_ sort of be avoided. It's the sort of value we could calculate on the fly, perhaps more easily... and we can cache that value as and when, as well. But... ooh, yikes, if we have to calculate it based on EVERY transaction, of which there could be thousands for a single account... that could be more of a problem. So...

...some accounts should store cached values...

...and others shouldn't; higher up the relationship tree, their values should be calculated on the fly from those caches.

That... seems reasonable and limits the insertion N+1 problem considerably.

All of which means...

_This is possible?_ I mean... _It's possible to achieve the goals of Marmalade within the principles of Unlabel?_

It seems I could do apps like...

- Unlabel Todos
- Unlabel Calendar
- Unlabel Accounting

...and a backend service like...

- Unlabel Graph

...the combination of all of which could go together to make...

- Marmalade

Yeah. This is gonna work.

That just leaves the user account question. I already plan to create a separate auth service that's open source and white labelled, _Unlabel Auth_... but I do maintain... I want these apps deployable in isolation. End users shouldn't have to deploy _Unlabel Auth_ as well, if all they want is _Unlabel Todos_, which means _Unlabel Todos_ has gotta have auth built in. So, same question: Is there a way I can have my cake here, and also eat it?

Let me tell you about my preferred authentication strategy, JSON Web Tokens or JWTs.

JWTs are great. I started using them years ago after running up against the limitations of Rails developers' preferred authentication solution, Devise. Devise is fantastic as a goto for same-origin authentication, since it uses cookies to store your authenticated session (cookies are sent with every request to the same domain, or "same-origin"). The problem with Devise is when you want to authenticate a mobile or desktop app, or any application on a separate domain. You don't have access to cookies, so the authentication strategy doesn't work. Enter JWTs. JWTs can be stored in session storage and sent manually with every request. They contain information about the authenticated user and a hashed signature which represents that same information that has been encoded with a known secret token. By validating that the signature matches the raw information using the token, we know it's from a legitimate source. This means that the JWT can be distributed. It can be sent to mobile and desktop apps, other domains, and used to authenticate our users anywhere.

Great! But how does that help us?

It doesn't. Our applications will still need to store the user for initial authentication. And if a developer wants to first deploy one application - maybe _Unlabel Todos_ - and then decides to deploy another - _Unlabel Calendar_ - and finally sometime later decides to start using _Unlabel Auth_, well then we have a problem... First one, then two, then three separate locations where independent authentication is happening.

We can share a secret token between them, so that it looks like all three are the same authenticating platform... but they still... aren't. They still maintain different sets of user records.

Imagine the scenario: A user signs up for one with their email address, then another... They have two accounts there that aren't necessarily tied together. The way in which we might tie these together is by using one service to authenticate the other; like how you can use Facebook to sign in or sign up with so many sites now. That part's not so difficult if we treat them as independent properties... listing, for instance, a "Authenticate with Unlabel Todos" option AND an "Authenticate with Unlabel Calendar" option. But you can see how that gets out of hand as we try to grow our platform using other additional services. Hence, y'know, _Unlabel Auth_ as the goto central station of authentication.

So... this doesn't sound ideal, but maybe our solution is this...

1. You can sign up individually at _App One_, whatever that may be...
2. You can sign up individually at _App Two_, whatever that may be (these accounts are not linked)...
3. You can sign up at _Unlabel Auth_ using authentication from either _App One_ or _App Two_...
4. You can connect the second of those accounts to _Unlabel Auth_ as well.

This would mean that developers could introduce the centralised authentication solution _later_ if they want integrated accounts, and they don't lose out by doing it the _other_ way first. The only real downsides being... they have to tell their _Unlabel Auth_ deployment what those other services they already have deployed are... and users do have to authenticate manually for each service they're already connected to.

Honestly, it makes integrating all of these very difficult. And in this hypothetical scenario, we haven't even gotten to the idea of introducing the graph. What if a user of our _Unlabel_ platform wants authentication to be well-integrated like this... but they don't want the broader integration that could be introduced? Might that be a problem? ... Probably not; advanced integration really would be a more complicated thing anyways, likely making use of APIs - I imagine it being difficult to activate unless they really wanted to.

So...

That all sounds reasonable to me.

I already know some things about my approach that I haven't mentioned here, like... we'll be creating Rails engines and Ruby Gems where possible, so that application logic is shareable and reusable. For example, I've talked about having it that all of these apps feature authentication. What that means in my mind is that _Unlabel Auth_ is not just an app, but is also the name for an associated Ruby Gem that another project can install and use. This way I'm not rewriting authentication logic, and I am only maintaining the one authentication solution. Other apps simply need a periodic update to keep up with it.

This _engine_ approach is something I am thinking I'll more broadly apply, meaning that - say - _Unlabel Todos_ and _Unlabel Calendar_ could be based on separate engines as well, and distributed as Gems. That way, if someone wanted to build a single application from these engines (much like I intend to do with Marmalade), that's entirely possible... You've got options. You can either individually deploy and manage a set of microservices, or you can install them all in one place as a single monolith. And if you change your mind..? Well... if I do this properly, that should be a pretty easy migration. We'll be using UUIDs rather than sequential IDs so that ID conflicts are avoided, and because the monolith and microservice architectures will be based on the same engines... they should very happily receive data from one another. Even replacing one part of a database monolith with an alternative API-approach... should go... simply enough... assuming I document that well.

In fact, I remain likely to take the monolith approach with _Marmalade_. This entire discussion was about whether or not I can avoid that, and we've learned that I could. But I think initially I would like to take the monolith approach anyway. It's proof of concept... and if I wind up breaking it apart into microservices, then it's proof of that concept too.

All right...

All right.

I have got a lot of work to do.